use std::time::{Duration, Instant};

use crate::{
    book::probe_opening_book,
    evaluate::{Eval, INFINITY, evaluate},
    movegen::{MoveList, generate_legal, generate_legal_noisy},
    position::{Position, StateInfo},
    types::Move,
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

    let noisy_moves = generate_legal_noisy(pos, &mut MoveList::new());

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
    depth: i32,
    ply: i32,
    mut alpha: Eval,
    beta: Eval,
    ctx: &mut SearchContext,
) -> Eval {
    ctx.stats.nodes += 1;
    if ctx.should_stop() {
        return 0;
    }

    let moves = generate_legal(pos, &mut MoveList::new());

    if moves.is_empty() {
        if pos.checkers.is_empty() {
            return 0; // stalemate
        } else {
            return -INFINITY + ply; // checkmate
        }
    }

    if pos.halfmove_clock >= 100 {
        return 0;
    }

    if depth == 0 {
        return q_search(pos, ply, alpha, beta, ctx);
    }

    let mut best_score = -INFINITY;

    for &mv in moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);

        let score = -negamax(pos, depth - 1, ply + 1, -beta, -alpha, ctx);

        pos.undo_move(mv, &state);

        if score > best_score {
            best_score = score;

            if score > alpha {
                alpha = score;
            }
        }

        if score >= beta {
            break;
        }
    }

    best_score
}

fn search_root(pos: &mut Position, depth: i32, ctx: &mut SearchContext) -> SearchResult {
    let moves = generate_legal(pos, &mut MoveList::new());

    let mut best_score = -INFINITY;
    let mut best_move = Move::NULL;

    for &mv in moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);

        let score = -negamax(pos, depth - 1, 1, -INFINITY, INFINITY, ctx);

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
    max_depth: i32,
    ctx: &mut SearchContext,
    start: Instant,
) -> SearchResult {
    let mut best = SearchResult::new();

    for depth in 1..=max_depth {
        if ctx.should_stop() {
            break;
        }

        // We only want to keep results from completed iterations
        let current = search_root(pos, depth, ctx);

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

pub fn search(pos: &mut Position, limits: SearchLimits) -> SearchResult {
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

    let mut result = iterative_deepening(pos, limits.max_depth, &mut ctx, start);

    if result.best_move == Move::NULL {
        result.best_move = fallback;
    }

    result
}
