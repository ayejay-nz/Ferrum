use std::time::{Duration, Instant};

use crate::{
    evaluate::{Eval, INFINITY, evaluate},
    movegen::{MoveList, generate_legal},
    position::{Position, StateInfo},
    types::Move,
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
        return evaluate(pos);
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

        best = current;
    }

    best
}

fn first_legal_move(pos: &Position) -> Move {
    let moves = generate_legal(pos, &mut MoveList::new());
    moves.as_slice().first().copied().unwrap_or(Move::NULL)
}

pub fn search(pos: &mut Position, limits: SearchLimits) -> SearchResult {
    let stop_at = limits.move_time.map(|t| Instant::now() + t);

    let mut ctx = SearchContext {
        stats: SearchStats::new(),
        stop_at,
        stopped: false,
    };

    let fallback = first_legal_move(pos);

    let mut result = iterative_deepening(pos, limits.max_depth, &mut ctx);

    if result.best_move == Move::NULL {
        result.best_move = fallback;
    }

    result
}
