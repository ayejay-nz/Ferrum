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

impl OrderingTables {
    pub fn new() -> Self {
        Self {
            killers: [[Move::NULL; 2]; MAX_PLY],
            history: [[[0; 64]; 64]; 2],
        }
    }
}

fn update_killers(killers: &mut [[Move; 2]; MAX_PLY], mv: Move, ply: usize) {
    if ply >= MAX_PLY {
        return;
    }

    // Insert killer move, ensuring it is unique at the ply level
    if killers[ply][0] != mv {
        killers[ply][1] = killers[ply][0];
        killers[ply][0] = mv;
    }
}

fn update_history(history: &mut [[[i32; 64]; 64]; 2], mv: Move, depth: i32, side: usize) {
    let from = mv.from().idx();
    let to = mv.to().idx();
    let bonus = depth * depth;
    let weight = history[side][from][to] + bonus;

    history[side][from][to] = weight.min(MAX_HISTORY);
}

#[inline(always)]
fn captured_piece(pos: &Position, mv: Move) -> Option<Piece> {
    if mv.is_ep_capture() {
        Some(Piece::Pawn)
    } else {
        pos.mailbox.piece_at(mv.to())
    }
}

#[inline(always)]
fn is_mate_score(score: Eval) -> bool {
    score.abs() >= INFINITY - 1000
}

#[inline(always)]
fn move_order_score(
    pos: &Position,
    ordering: &OrderingTables,
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
    if let Some(victim) = captured_piece(pos, mv) {
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
    if mv == ordering.killers[ply][0] {
        return KILLER1_BONUS;
    }
    if mv == ordering.killers[ply][1] {
        return KILLER2_BONUS;
    }

    // Sort history moves
    let side = pos.side_to_move.idx();
    ordering.history[side][mv.from().idx()][mv.to().idx()]
}

fn order_moves(
    pos: &Position,
    ordering: &OrderingTables,
    ply: usize,
    moves: &mut MoveList,
    pv_move: Move,
    tt_move: Move,
) {
    moves.as_mut_slice().sort_unstable_by_key(|&mv| {
        Reverse(move_order_score(pos, ordering, ply, mv, pv_move, tt_move))
    });
}

fn is_repetition(pos: &Position, rep_history: &[ZKey]) -> bool {
    // We only need to find one more instance of position in history to consider it a draw
    let mut back = 2usize;
    let limit = pos.halfmove_clock as usize;

    let max_back = limit.min(rep_history.len() - 1);

    while back <= max_back {
        if let Some(&prev_key) = rep_history.get(rep_history.len() - 1 - back) {
            if prev_key == pos.zkey {
                return true;
            }
        }
        back += 2;
    }

    false
}

fn q_search(
    pos: &mut Position,
    rep_history: &mut Vec<ZKey>,
    ordering: &OrderingTables,
    ply: i32,
    mut alpha: Eval,
    beta: Eval,
    ctx: &mut SearchContext,
) -> Eval {
    ctx.stats.nodes += 1;
    if ctx.should_stop() {
        return 0;
    }

    if pos.halfmove_clock >= 100 {
        return 0;
    }
    if is_repetition(pos, rep_history) {
        return 0;
    }
    if pos.insufficient_material() {
        return 0;
    }

    // If in check, search all legal evasions
    if !pos.checkers.is_empty() {
        let moves = generate_legal(pos, &mut MoveList::new());

        if moves.is_empty() {
            return -INFINITY + ply;
        }

        let mut best_score = -INFINITY;

        for &mv in moves.as_slice() {
            let mut state = StateInfo::new();
            pos.make_move(mv, &mut state);
            rep_history.push(pos.zkey);

            let score = -q_search(pos, rep_history, ordering, ply + 1, -beta, -alpha, ctx);

            pos.undo_move(mv, &state);
            rep_history.pop();

            if ctx.stopped {
                return 0;
            }

            if score >= beta {
                return score;
            }
            if score > best_score {
                best_score = score;
            }
            if score > alpha {
                alpha = score;
            }
        }

        return best_score;
    }

    // Stand pat - Assume at least one move can either match or beat the lower score bound
    // Based on null move observation: assumes we are not in zugzwan
    let stand_pat = evaluate(pos);
    let mut best_score = stand_pat;

    if stand_pat >= beta {
        return stand_pat;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    let mut noisy_moves = generate_legal_noisy(pos, &mut MoveList::new());
    order_moves(
        pos,
        ordering,
        ply as usize,
        &mut noisy_moves,
        Move::NULL,
        Move::NULL,
    );

    for &mv in noisy_moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);
        rep_history.push(pos.zkey);

        let score = -q_search(pos, rep_history, ordering, ply + 1, -beta, -alpha, ctx);

        pos.undo_move(mv, &state);
        rep_history.pop();

        if ctx.stopped {
            return 0;
        }

        if score >= beta {
            return score;
        }
        if score > best_score {
            best_score = score;
        }
        if score > alpha {
            alpha = score;
        }
    }

    best_score
}

fn negamax(
    pos: &mut Position,
    tt: &mut TranspositionTable,
    rep_history: &mut Vec<ZKey>,
    ordering: &mut OrderingTables,
    depth: i32,
    ply: i32,
    mut alpha: Eval,
    mut beta: Eval,
    ctx: &mut SearchContext,
) -> Eval {
    ctx.stats.nodes += 1;
    if ctx.should_stop() {
        return 0;
    }

    if pos.halfmove_clock >= 100 {
        return 0;
    }
    if is_repetition(pos, rep_history) {
        return 0;
    }
    if pos.insufficient_material() {
        return 0;
    }

    let alpha_orig = alpha;
    let beta_orig = beta;
    let mut tt_move = Move::NULL;

    // Check transposition table
    if let Some(hit) = tt.probe(pos.zkey) {
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

    let mut moves = generate_legal(pos, &mut MoveList::new());

    if moves.is_empty() {
        if pos.checkers.is_empty() {
            return 0; // stalemate
        } else {
            return -INFINITY + ply; // checkmate
        }
    }

    if depth == 0 {
        return q_search(pos, rep_history, ordering, ply, alpha, beta, ctx);
    }

    let mut best_score = -INFINITY;
    let mut best_move = Move::NULL;

    order_moves(pos, ordering, ply as usize, &mut moves, Move::NULL, tt_move);
    for &mv in moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);
        rep_history.push(pos.zkey);

        let score = -negamax(
            pos,
            tt,
            rep_history,
            ordering,
            depth - 1,
            ply + 1,
            -beta,
            -alpha,
            ctx,
        );

        pos.undo_move(mv, &state);
        rep_history.pop();

        if score > best_score {
            best_score = score;
            best_move = mv;

            if score > alpha {
                alpha = score;
            }
        }

        if score >= beta {
            if mv.is_quiet() {
                update_killers(&mut ordering.killers, mv, ply as usize);
                update_history(&mut ordering.history, mv, depth, pos.side_to_move.idx());
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

    tt.store(
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

fn search_root(
    pos: &mut Position,
    tt: &mut TranspositionTable,
    rep_history: &mut Vec<ZKey>,
    ordering: &mut OrderingTables,
    pv_move: Move,
    depth: i32,
    ctx: &mut SearchContext,
) -> SearchResult {
    let mut moves = generate_legal(pos, &mut MoveList::new());

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

    let mut best_score = -INFINITY;
    let mut best_move = Move::NULL;

    let mut alpha = -INFINITY;
    let mut beta = INFINITY;
    let mut tt_move = Move::NULL;

    // Check transposition table
    if let Some(hit) = tt.probe(pos.zkey) {
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

    order_moves(pos, ordering, 0, &mut moves, pv_move, tt_move);

    for &mv in moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);
        rep_history.push(pos.zkey);

        let score = -negamax(
            pos,
            tt,
            rep_history,
            ordering,
            depth - 1,
            1,
            -beta,
            -alpha,
            ctx,
        );

        pos.undo_move(mv, &state);
        rep_history.pop();

        if score > best_score {
            best_score = score;
            best_move = mv;
        }
        if score > alpha {
            alpha = score;
        }
    }

    // Store values in transposition table
    // Use full window for now, but will need to stop
    // using Exact when adding aspiration windows
    tt.store(
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
    pos: &mut Position,
    tt: &mut TranspositionTable,
    rep_history: &mut Vec<ZKey>,
    ordering: &mut OrderingTables,
    max_depth: i32,
    ctx: &mut SearchContext,
    start: Instant,
) -> SearchResult {
    let mut best = SearchResult::new();

    for depth in 1..=max_depth {
        if ctx.should_stop() {
            break;
        }

        let current = search_root(pos, tt, rep_history, ordering, best.best_move, depth, ctx);

        // We only want to keep results from completed iterations
        if ctx.stopped {
            break;
        }

        uci::emit_uci_info(&current, ctx, start);
        best = current;
    }

    best
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
) -> SearchResult {
    let start = Instant::now();

    let mut rep_history = history.to_vec();
    let mut ordering = OrderingTables::new();

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

    let stop_at = limits.move_time.map(|t| start + t);

    let mut ctx = SearchContext {
        stats: SearchStats::new(),
        stop_at,
        stopped: false,
    };

    let fallback = first_legal_move(pos);

    let mut result = iterative_deepening(
        pos,
        tt,
        &mut rep_history,
        &mut ordering,
        limits.max_depth,
        &mut ctx,
        start,
    );

    if result.best_move == Move::NULL {
        result.best_move = fallback;
    }

    tt.increment_age();

    result
}
