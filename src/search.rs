use std::time::Instant;

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

fn negamax(
    pos: &mut Position,
    depth: i32,
    ply: i32,
    mut alpha: Eval,
    beta: Eval,
    stats: &mut SearchStats,
) -> Eval {
    stats.nodes += 1;

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

        let score = -negamax(pos, depth - 1, ply + 1, -beta, -alpha, stats);

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

fn search_root(pos: &mut Position, depth: i32, stats: &mut SearchStats) -> SearchResult {
    let moves = generate_legal(pos, &mut MoveList::new());

    let mut best_score = -INFINITY;
    let mut best_move = Move::NULL;

    for &mv in moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);

        let score = -negamax(pos, depth - 1, 1, -INFINITY, INFINITY, stats);

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
    stats: &mut SearchStats,
) -> SearchResult {
    let mut best = SearchResult::new();

    for depth in 1..=max_depth {
        best = search_root(pos, depth, stats);
    }

    best
}

pub fn search(pos: &mut Position, max_depth: i32) -> SearchResult {
    let start = Instant::now();
    let mut stats = SearchStats::new();

    let result = iterative_deepening(pos, max_depth, &mut stats);

    let secs = start.elapsed().as_secs_f64();
    let nps = if secs > 0.0 {
        (stats.nodes as f64 / secs) as u64
    } else {
        0
    };

    println!(
        "
    best move: {}{}, evaluation: {}, depth: {}, nodes: {}, nps: {}, time elapsed: {}s",
        result.best_move.from(),
        result.best_move.to(),
        result.score,
        result.depth,
        stats.nodes,
        nps,
        secs
    );

    result
}
