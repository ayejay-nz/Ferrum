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
fn move_order_score(pos: &Position, mv: Move, pv_move: Move, tt_move: Move) -> i32 {
    // Always want PV move to be first, followed by TT-move
    if mv == pv_move {
        return 1001;
    }
    if mv == tt_move {
        return 1000;
    }

    let attacker = pos.mailbox.piece_at(mv.from()).unwrap();
    let mut score = 0;

    // MVV-LVA move ordering
    if let Some(victim) = captured_piece(pos, mv) {
        score += 128 + 8 * (victim as i32 + 1) - attacker as i32
    }

    // Sort promotions in the order which they most commonly occur
    if let Some(promo) = mv.promotion_piece() {
        match promo {
            Piece::Queen => score += 4,
            Piece::Knight => score += 3,
            Piece::Rook => score += 2,
            Piece::Bishop => score += 1,
            _ => unreachable!(),
        }
    }

    score
}

fn order_moves(pos: &Position, moves: &mut MoveList, pv_move: Move, tt_move: Move) {
    moves
        .as_mut_slice()
        .sort_unstable_by_key(|&mv| Reverse(move_order_score(pos, mv, pv_move, tt_move)));
}

fn q_search(
    pos: &mut Position,
    ply: i32,
    mut alpha: Eval,
    beta: Eval,
    ctx: &mut SearchContext,
) -> Eval {
    ctx.stats.nodes += 1;
    if ctx.should_stop() {
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

            let score = -q_search(pos, ply + 1, -beta, -alpha, ctx);

            pos.undo_move(mv, &state);

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
    order_moves(pos, &mut noisy_moves, Move::NULL, Move::NULL);

    for &mv in noisy_moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);

        let score = -q_search(pos, ply + 1, -beta, -alpha, ctx);

        pos.undo_move(mv, &state);

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
        return q_search(pos, ply, alpha, beta, ctx);
    }

    let mut best_score = -INFINITY;
    let mut best_move = Move::NULL;

    order_moves(pos, &mut moves, Move::NULL, tt_move);
    for &mv in moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);

        let score = -negamax(pos, tt, depth - 1, ply + 1, -beta, -alpha, ctx);

        pos.undo_move(mv, &state);

        if score > best_score {
            best_score = score;
            best_move = mv;

            if score > alpha {
                alpha = score;
            }
        }

        if score >= beta {
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

    let tt_move = tt.probe(pos.zkey).map_or(Move::NULL, |hit| hit.mv);
    order_moves(pos, &mut moves, pv_move, tt_move);

    let mut best_score = -INFINITY;
    let mut best_move = Move::NULL;

    for &mv in moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);

        let score = -negamax(pos, tt, depth - 1, 1, -INFINITY, INFINITY, ctx);

        pos.undo_move(mv, &state);

        if score > best_score {
            best_score = score;
            best_move = mv;
        }
    }

    SearchResult {
        best_move,
        score: best_score,
        depth,
    }
}

fn iterative_deepening(
    pos: &mut Position,
    tt: &mut TranspositionTable,
    max_depth: i32,
    ctx: &mut SearchContext,
    start: Instant,
) -> SearchResult {
    let mut best = SearchResult::new();

    for depth in 1..=max_depth {
        if ctx.should_stop() {
            break;
        }

        let current = search_root(pos, tt, best.best_move, depth, ctx);

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
    limits: SearchLimits,
) -> SearchResult {
    let start = Instant::now();

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

    let mut result = iterative_deepening(pos, tt, limits.max_depth, &mut ctx, start);

    if result.best_move == Move::NULL {
        result.best_move = fallback;
    }

    tt.increment_age();

    result
}
