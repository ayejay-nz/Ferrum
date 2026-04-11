use core::f64;
use rayon::prelude::*;
use std::cmp::Reverse;

use crate::movegen::{MoveList, generate_legal};
use crate::{
    evaluate::{Eval, INFINITY, evaluate_with},
    movegen::generate_legal_noisy,
    position::{Position, StateInfo},
    search::OrderingTables,
    tune::Params,
    tuning::types::Sample,
    types::Colour,
};

fn texel_qsearch(pos: &mut Position, params: &Params, mut alpha: Eval, beta: Eval) -> Eval {
    if pos.halfmove_clock >= 100 || pos.insufficient_material() {
        return 0;
    }

    let in_check = !pos.checkers.is_empty();

    let mut best_score = if in_check {
        -INFINITY
    } else {
        let stand_pat = evaluate_with(pos, params);

        if stand_pat >= beta {
            return stand_pat;
        }

        alpha = alpha.max(stand_pat);
        stand_pat
    };

    let mut moves = if in_check {
        generate_legal(pos, &mut MoveList::new())
    } else {
        generate_legal_noisy(pos, &mut MoveList::new())
    };

    let ordering = OrderingTables::new();
    moves
        .as_mut_slice()
        .sort_unstable_by_key(|&mv| Reverse(ordering.score_noisy(pos, mv)));

    for &mv in moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);

        let score = -texel_qsearch(pos, params, -beta, -alpha);

        pos.undo_move(mv, &state);

        if score >= beta {
            return score;
        }

        best_score = best_score.max(score);
        alpha = alpha.max(score);
    }

    best_score
}

fn texel_root_qsearch(pos: &mut Position, params: &Params) -> Eval {
    let score = texel_qsearch(pos, params, -INFINITY, INFINITY);

    return if pos.side_to_move == Colour::White {
        score
    } else {
        -score
    };
}

fn sigmoid(k: f64, qscore: i32) -> f64 {
    1.0 / (1.0 + 10.0_f64.powf(-k * qscore as f64 / 400.0)) as f64
}

fn sample_error(sample: &Sample, params: &Params, k: f64) -> f64 {
    let mut pos = sample.pos;
    let result = sample.result.to_value();

    let qscore = texel_root_qsearch(&mut pos, &params);

    // Calculate evaluation error
    let diff = result - sigmoid(k, qscore);
    diff * diff
}

pub fn loss(samples: &[Sample], params: &Params, k: f64) -> f64 {
    let total: f64 = samples.par_iter().map(|s| sample_error(s, params, k)).sum();

    total / samples.len() as f64
}

pub fn fit_k(samples: &[Sample], params: &Params) -> f64 {
    let mut best_k = 0.1;
    let mut best_loss = f64::INFINITY;

    let mut k = 0.0;
    while k <= 2.0 {
        let err = loss(samples, params, k);
        if err < best_loss {
            best_loss = err;
            best_k = k;
        }
        k += 0.001;
    }

    best_k
}
