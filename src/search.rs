use std::{
    cmp::Reverse,
    time::{Duration, Instant},
};

use crate::{
    book::probe_opening_book,
    evaluate::{Eval, INFINITY, evaluate},
    movegen::{MoveList, generate_legal, generate_legal_noisy},
    position::{Position, StateInfo},
    tt::{BoundType, TranspositionTable},
    types::{Move, Piece},
    uci,
    zobrist::ZKey,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum NodeType {
    PV,
    NonPV,
}

impl NodeType {
    fn is_pv(self) -> bool {
        !(self == NodeType::NonPV)
    }
}

pub struct SearchStats {
    pub nodes: u64,
}

impl SearchStats {
    fn new() -> Self {
        Self { nodes: 0 }
    }
}

pub struct SearchResult {
    pub best_move: Move,
    pub score: Eval,
    pub depth: i32,
}

impl SearchResult {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            best_move: Move::NULL,
            score: -INFINITY,
            depth: 0,
        }
    }
}

pub struct SearchLimits {
    pub max_depth: i32,
    pub move_time: Option<Duration>,
}

pub struct SearchContext {
    pub stats: SearchStats,
    pub stop_at: Option<Instant>,
    pub stopped: bool,
}

impl SearchContext {
    #[inline(always)]
    fn should_stop(&mut self) -> bool {
        if self.stopped {
            return true;
        }

        // Only check stop time every 2048 nodes
        if self.stats.nodes & 2047 != 0 {
            return false;
        }

        if let Some(stop_at) = self.stop_at {
            if Instant::now() >= stop_at {
                self.stopped = true;
            }
        }

        self.stopped
    }
}

pub struct Searcher<'a> {
    tt: &'a mut TranspositionTable,
    rep_history: &'a mut Vec<ZKey>,
    ordering: OrderingTables,
    ctx: SearchContext,
}

const PV_BONUS: i32 = 1_000_000;
const TT_BONUS: i32 = 900_000;
const CAPTURE_BASE: i32 = 800_000;
const PROMOTION_BASE: i32 = 700_000;
const KILLER1_BONUS: i32 = 600_000;
const KILLER2_BONUS: i32 = 599_000;

const MAX_PLY: usize = 256;
const MAX_HISTORY: i32 = 16384;

pub struct OrderingTables {
    killers: [[Move; 2]; MAX_PLY],
    history: [[[i32; 64]; 64]; 2],
}

#[inline(always)]
fn is_mate_score(score: Eval) -> bool {
    score.abs() >= INFINITY - 1000
}

/// Check if the provided conditions allow for LMR
///
/// Don't perform LMR on:
/// - Captures and promotions
/// - Moves while in check
/// - Moves which give check
/// - Killer moves
/// - Depth is too low (depth < 3)
#[inline(always)]
fn can_lmr(
    pos: &Position,
    killers: &[[Move; 2]; MAX_PLY],
    mv: Move,
    in_check: bool,
    depth: i32,
    ply: i32,
) -> bool {
    if depth < 3 {
        return false;
    }
    if mv.is_capture() || mv.is_promotion() {
        return false;
    }
    if in_check || !pos.checkers.is_empty() {
        return false;
    }
    if killers[ply as usize].contains(&mv) {
        return false;
    }

    true
}

#[inline(always)]
fn lmr_reduction(idx: usize) -> i32 {
    // Dont reduce on first 3 moves
    if idx < 3 {
        return 0;
    }
    if idx > 12 {
        return 2;
    }

    1
}

#[inline(always)]
fn draw_score(ply: i32) -> Eval {
    // If ply is even that means we are searching the root sides moves,
    // so a draw should be slightly punished, and vice versa
    const CONTEMPT: Eval = 10;

    if ply % 2 == 0 { -CONTEMPT } else { CONTEMPT }
}

impl OrderingTables {
    pub fn new() -> Self {
        Self {
            killers: [[Move::NULL; 2]; MAX_PLY],
            history: [[[0; 64]; 64]; 2],
        }
    }

    pub fn update_killers(&mut self, mv: Move, ply: usize) {
        if ply >= MAX_PLY {
            return;
        }

        // Insert killer move, ensuring it is unique at the ply level
        if self.killers[ply][0] != mv {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = mv;
        }
    }

    pub fn update_history(&mut self, mv: Move, depth: i32, side: usize) {
        let from = mv.from().idx();
        let to = mv.to().idx();
        let bonus = depth * depth;
        let weight = self.history[side][from][to] + bonus;

        self.history[side][from][to] = weight.min(MAX_HISTORY);
    }

    #[inline(always)]
    fn move_order_score(
        &self,
        pos: &Position,
        ply: usize,
        mv: Move,
        pv_move: Move,
        tt_move: Move,
    ) -> i32 {
        // Always want PV move to be first, followed by TT-move
        if mv == pv_move {
            return PV_BONUS;
        }
        if mv == tt_move {
            return TT_BONUS;
        }

        // MVV-LVA move ordering
        if let Some(victim) = pos.captured_piece(mv) {
            let attacker = pos.mailbox.piece_at(mv.from()).unwrap();
            let mut score = CAPTURE_BASE + 8 * (victim as i32 + 1) - attacker as i32;

            if let Some(promo) = mv.promotion_piece() {
                score += match promo {
                    Piece::Queen => 40,
                    Piece::Knight => 30,
                    Piece::Rook => 20,
                    Piece::Bishop => 10,
                    _ => unreachable!(),
                }
            }

            return score;
        }

        // Sort promotions in the order which they most commonly occur
        if let Some(promo) = mv.promotion_piece() {
            return PROMOTION_BASE
                + match promo {
                    Piece::Queen => 40,
                    Piece::Knight => 30,
                    Piece::Rook => 20,
                    Piece::Bishop => 10,
                    _ => unreachable!(),
                };
        }

        // Sort killers moves
        if mv == self.killers[ply][0] {
            return KILLER1_BONUS;
        }
        if mv == self.killers[ply][1] {
            return KILLER2_BONUS;
        }

        // Sort history moves
        let side = pos.side_to_move.idx();
        self.history[side][mv.from().idx()][mv.to().idx()]
    }

    pub fn order_moves(
        &self,
        pos: &Position,
        ply: usize,
        moves: &mut MoveList,
        pv_move: Move,
        tt_move: Move,
    ) {
        moves.as_mut_slice().sort_unstable_by_key(|&mv| {
            Reverse(self.move_order_score(pos, ply, mv, pv_move, tt_move))
        });
    }
}

impl<'a> Searcher<'a> {
    fn is_repetition(&self, pos: &Position) -> bool {
        // We only need to find one more instance of position in history to consider it a draw
        let mut back = 2usize;
        let limit = pos.halfmove_clock as usize;

        let max_back = limit.min(self.rep_history.len() - 1);

        while back <= max_back {
            if let Some(&prev_key) = self.rep_history.get(self.rep_history.len() - 1 - back) {
                if prev_key == pos.zkey {
                    return true;
                }
            }
            back += 2;
        }

        false
    }

    fn q_search(&mut self, pos: &mut Position, ply: i32, mut alpha: Eval, beta: Eval) -> Eval {
        self.ctx.stats.nodes += 1;
        if self.ctx.should_stop() {
            return 0;
        }

        // Check that position is not a forced draw
        if pos.halfmove_clock >= 100 || self.is_repetition(pos) || pos.insufficient_material() {
            return 0;
        }

        let in_check = !pos.checkers.is_empty();
        let mut state = StateInfo::new();
        state.set_from_position(pos);

        let mut best_score;
        let mut moves;

        // If we are in check, we cannot standpat and instead must search all evasions.
        // If we are not in check, we search all capturing moves, returning if we get
        // a beta cutoff or have searched all moves (i.e. reached a quiet position)
        if in_check {
            moves = generate_legal(pos, &mut MoveList::new());

            if moves.is_empty() {
                return -INFINITY + ply;
            }

            best_score = -INFINITY;
        } else {
            let stand_pat = evaluate(pos);

            if stand_pat >= beta {
                return stand_pat;
            }

            alpha = alpha.max(stand_pat);
            best_score = stand_pat;

            moves = generate_legal_noisy(pos, &mut MoveList::new());
        }

        self.ordering
            .order_moves(pos, ply as usize, &mut moves, Move::NULL, Move::NULL);

        // Search through all moves until a beta cutoff or none left
        for &mv in moves.as_slice() {
            pos.make_move(mv, &mut state);
            self.rep_history.push(pos.zkey);

            let score = -self.q_search(pos, ply + 1, -beta, -alpha);

            pos.undo_move(mv, &state);
            self.rep_history.pop();

            if score >= beta {
                return score;
            }

            best_score = best_score.max(score);
            alpha = alpha.max(score);
        }

        best_score
    }

    fn negamax(
        &mut self,
        pos: &mut Position,
        depth: i32,
        ply: i32,
        mut alpha: Eval,
        mut beta: Eval,
        node_type: NodeType,
    ) -> Eval {
        self.ctx.stats.nodes += 1;
        if self.ctx.should_stop() {
            return 0;
        }

        // Check that position is not a forced draw
        if pos.halfmove_clock >= 100 || self.is_repetition(pos) || pos.insufficient_material() {
            return 0;
        }

        // When depth is 0, go into quiescence search
        if depth <= 0 {
            return self.q_search(pos, ply, alpha, beta);
        }

        // Setup search information
        let us = pos.side_to_move;
        let alpha_orig = alpha;
        let beta_orig = beta;
        let mut tt_move = Move::NULL;

        // Check transposition table
        if let Some(hit) = self.tt.probe(pos.zkey) {
            tt_move = hit.mv;

            if hit.depth as i32 >= depth {
                let bound_type = hit.node_info.bound_type();

                // Normalise mating distance across different depths
                let mut score = hit.value as Eval;
                if is_mate_score(score) {
                    score += if score > 0 { -ply } else { ply };
                }

                match bound_type {
                    BoundType::Exact => return score,
                    BoundType::Upper => beta = beta.min(score),
                    BoundType::Lower => alpha = alpha.max(score),
                    _ => unreachable!(),
                }

                if alpha >= beta {
                    return score;
                }
            }
        }

        // Generate legal moves
        let mut moves = generate_legal(pos, &mut MoveList::new());

        // Handle checkmate/stalemate
        if moves.is_empty() {
            if pos.checkers.is_empty() {
                return 0; // stalemate
            } else {
                return -INFINITY + ply; // checkmate
            }
        }

        let in_check = !pos.checkers.is_empty();
        let is_pv_node = node_type.is_pv();

        let mut state = StateInfo::new();
        state.set_from_position(pos);

        // Perform Null Move Pruning when:
        // - The player is not in check
        // - The player doesn't just have pawns left
        // - Search depth is adequite to accidentally skip a zugzwang
        if !in_check && !pos.non_pawn_material(us).is_empty() && depth >= 3 {
            // Null move reduction
            let r = 2 + depth / 3;
            let null_depth = 0.max(depth - r);

            pos.make_null_move(&mut state);
            self.rep_history.push(pos.zkey);

            let v = -self.negamax(pos, null_depth, ply + 1, -beta, -beta + 1, NodeType::NonPV);

            pos.undo_null_move(&state);
            self.rep_history.pop();

            // Null Move Observation lets us skip remaining moves
            if v >= beta {
                return v;
            }
        }

        self.ordering
            .order_moves(pos, ply as usize, &mut moves, Move::NULL, tt_move);

        let mut best_score = -INFINITY;
        let mut best_move = Move::NULL;

        // Iterate over all ordered legal moves until beta cutoff or none left
        for (idx, &mv) in moves.as_slice().iter().enumerate() {
            pos.make_move(mv, &mut state);
            self.rep_history.push(pos.zkey);

            let mut score = -INFINITY;
            let new_depth = depth - 1;

            let nw_alpha = -(alpha + 1);
            let nw_beta = -alpha;

            // Perform LMR and PVS
            if can_lmr(pos, &self.ordering.killers, mv, in_check, depth, ply) && idx > 0 {
                let r = lmr_reduction(idx);
                let d = new_depth - r;

                // Search using null window at reduced depth
                score = -self.negamax(pos, d, ply + 1, nw_alpha, nw_beta, NodeType::NonPV);

                // LMR fails high so we may have missed something,
                // We perform full depth search using null window
                if score > alpha {
                    score =
                        -self.negamax(pos, new_depth, ply + 1, nw_alpha, nw_beta, NodeType::NonPV);
                }
            }
            // Perform a full depth search on null window when LMR is skipped
            else if !is_pv_node || idx > 0 {
                score = -self.negamax(pos, new_depth, ply + 1, nw_alpha, nw_beta, NodeType::NonPV);
            }

            // Search the full window at full depth for PV nodes
            // The first move always get full depth as that is PV move from ordering
            // If a later move raised alpha without causing a beta cutoff,
            // we are also interested in searching full depth
            if is_pv_node && (idx == 0 || (alpha < score && score < beta)) {
                score = -self.negamax(pos, new_depth, ply + 1, -beta, -alpha, NodeType::PV);
            }

            pos.undo_move(mv, &state);
            self.rep_history.pop();

            if score > best_score {
                best_score = score;
                best_move = mv;

                alpha = alpha.max(score);
            }

            if score >= beta {
                if mv.is_quiet() {
                    self.ordering.update_killers(mv, ply as usize);
                    self.ordering.update_history(mv, depth, us.idx());
                }
                break;
            }
        }

        // Store values in transposition table
        let bound;
        if best_score <= alpha_orig {
            bound = BoundType::Upper;
        } else if best_score >= beta_orig {
            bound = BoundType::Lower;
        } else {
            bound = BoundType::Exact;
        }

        // Normalise mating score before storing
        let mut value = best_score;
        if is_mate_score(value) {
            value += if value > 0 { ply } else { -ply };
        }

        self.tt.store(
            pos.zkey,
            depth as u8,
            best_move,
            bound,
            false,
            value as i16,
            evaluate(pos) as i16,
        );

        best_score
    }

    fn search_root(&mut self, pos: &mut Position, pv_move: Move, depth: i32) -> SearchResult {
        let mut moves = generate_legal(pos, &mut MoveList::new());

        // Check for mates
        if moves.is_empty() {
            return SearchResult {
                best_move: Move::NULL,
                score: if pos.checkers.is_empty() {
                    0
                } else {
                    -INFINITY
                },
                depth,
            };
        }

        // Initialise search information
        let mut best_score = -INFINITY;
        let mut best_move = Move::NULL;

        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        let mut tt_move = Move::NULL;

        // Check transposition table
        if let Some(hit) = self.tt.probe(pos.zkey) {
            tt_move = hit.mv;

            if hit.depth as i32 >= depth {
                let bound_type = hit.node_info.bound_type();
                let score = hit.value as Eval;

                match bound_type {
                    BoundType::Exact => {
                        alpha = score;
                        best_score = score;
                        best_move = tt_move;
                    }
                    BoundType::Upper => beta = beta.min(score),
                    BoundType::Lower => alpha = alpha.max(score),
                    _ => unreachable!(),
                }
            }
        }

        self.ordering
            .order_moves(pos, 0, &mut moves, pv_move, tt_move);
        let mut state = StateInfo::new();

        for &mv in moves.as_slice() {
            pos.make_move(mv, &mut state);
            self.rep_history.push(pos.zkey);

            let score = -self.negamax(pos, depth - 1, 1, -beta, -alpha, NodeType::PV);

            pos.undo_move(mv, &state);
            self.rep_history.pop();

            if score > best_score {
                best_score = score;
                best_move = mv;
            }

            alpha = alpha.max(score);
        }

        // Store values in transposition table
        // Use full window for now, but will need to stop
        // using Exact when adding aspiration windows
        self.tt.store(
            pos.zkey,
            depth as u8,
            best_move,
            BoundType::Exact,
            false,
            best_score as i16,
            evaluate(pos) as i16,
        );

        SearchResult {
            best_move,
            score: best_score,
            depth,
        }
    }

    fn iterative_deepening(
        &mut self,
        pos: &mut Position,
        max_depth: i32,
        start: Instant,
    ) -> SearchResult {
        let mut best = SearchResult::new();

        for depth in 1..=max_depth {
            if self.ctx.should_stop() {
                break;
            }

            let current = self.search_root(pos, best.best_move, depth);

            // We only want to keep results from completed iterations
            if self.ctx.stopped {
                break;
            }

            uci::emit_uci_info(&current, &self.ctx, start);
            best = current;
        }

        best
    }
}

fn first_legal_move(pos: &Position) -> Move {
    let moves = generate_legal(pos, &mut MoveList::new());
    moves.as_slice().first().copied().unwrap_or(Move::NULL)
}

pub fn search(
    pos: &mut Position,
    tt: &mut TranspositionTable,
    history: &[ZKey],
    limits: SearchLimits,
    use_book: bool,
) -> SearchResult {
    let start = Instant::now();

    let mut rep_history = history.to_vec();

    // Probe opening book for best move
    if use_book {
        if let Some(book_move) = probe_opening_book(pos) {
            uci::emit_uci_info(
                &SearchResult {
                    best_move: book_move,
                    score: 0,
                    depth: 0,
                },
                &SearchContext {
                    stats: SearchStats { nodes: 0 },
                    stop_at: Some(Instant::now()),
                    stopped: true,
                },
                start,
            );
            return SearchResult {
                best_move: book_move,
                score: 0,
                depth: 0,
            };
        }
    }

    let stop_at = limits.move_time.map(|t| start + t);
    let fallback = first_legal_move(pos);

    let mut searcher = Searcher {
        tt,
        rep_history: &mut rep_history,
        ordering: OrderingTables::new(),
        ctx: SearchContext {
            stats: SearchStats::new(),
            stop_at,
            stopped: false,
        },
    };

    let mut result = searcher.iterative_deepening(pos, limits.max_depth, start);

    if result.best_move == Move::NULL {
        result.best_move = fallback;
    }

    searcher.tt.increment_age();

    result
}
