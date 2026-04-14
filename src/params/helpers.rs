use std::array;

use crate::{evaluate::Score, params::types::ParamBounds};

pub fn push_score(out: &mut Vec<i32>, s: Score) {
    out.push(s.mg);
    out.push(s.eg);
}

pub fn push_score_array<const N: usize>(out: &mut Vec<i32>, arr: &[Score; N]) {
    for &s in arr {
        push_score(out, s);
    }
}

pub fn push_pawn_pst(out: &mut Vec<i32>, pst: &[Score; 64]) {
    for (i, &s) in pst.iter().enumerate() {
        // Skip 1st and 8th rank
        if !(8..56).contains(&i) {
            continue;
        }
        out.push(s.mg);
        out.push(s.eg);
    }
}

pub fn next_score<I: Iterator<Item = i32>>(it: &mut I) -> Score {
    Score {
        mg: it.next().unwrap(),
        eg: it.next().unwrap(),
    }
}

pub fn next_score_array<const N: usize, I: Iterator<Item = i32>>(it: &mut I) -> [Score; N] {
    array::from_fn(|_| next_score(it))
}

pub fn next_pawn_pst<I: Iterator<Item = i32>>(it: &mut I, base: &[Score; 64]) -> [Score; 64] {
    let mut pst = *base;

    for (i, sq) in pst.iter_mut().enumerate() {
        // Skip 1st and 8th rank
        if !(8..56).contains(&i) {
            continue;
        }

        *sq = Score {
            mg: it.next().unwrap(),
            eg: it.next().unwrap(),
        }
    }

    pst
}

pub fn make_nondecreasing<const N: usize>(arr: &mut [Score; N]) {
    for i in 1..N {
        arr[i].mg = arr[i].mg.max(arr[i - 1].mg);
        arr[i].eg = arr[i].eg.max(arr[i - 1].eg);
    }
}

pub fn make_nonincreasing<const N: usize>(arr: &mut [Score; N]) {
    for i in 1..N {
        arr[i].mg = arr[i].mg.min(arr[i - 1].mg);
        arr[i].eg = arr[i].eg.min(arr[i - 1].eg);
    }
}

pub fn normalise_mean_zero<const N: usize>(base: &mut Score, arr: &mut [Score; N]) {
    let mean_mg = arr.iter().map(|s| s.mg).sum::<i32>() / N as i32;
    let mean_eg = arr.iter().map(|s| s.eg).sum::<i32>() / N as i32;

    for s in arr {
        s.mg -= mean_mg;
        s.eg -= mean_eg;
    }

    base.mg += mean_mg;
    base.eg += mean_eg;
}

fn shift_first_bucket_into_range<const N: usize>(arr: &mut [Score; N], lo: i32, hi: i32) {
    let shift_mg = if arr[0].mg < lo {
        lo - arr[0].mg
    } else if arr[0].mg > hi {
        hi - arr[0].mg
    } else {
        0
    };

    let shift_eg = if arr[0].eg < lo {
        lo - arr[0].eg
    } else if arr[0].eg > hi {
        hi - arr[0].eg
    } else {
        0
    };

    for s in arr {
        s.mg += shift_mg;
        s.eg += shift_eg;
    }
}

fn limit_subsequent_drop<const N: usize>(arr: &mut [Score; N], max_drop: i32, floor: i32) {
    for i in 1..N {
        let min_mg = (arr[i - 1].mg - max_drop).max(floor);
        let min_eg = (arr[i - 1].eg - max_drop).max(floor);

        arr[i].mg = arr[i].mg.clamp(min_mg, arr[i - 1].mg);
        arr[i].eg = arr[i].eg.clamp(min_eg, arr[i - 1].eg);
    }
}

pub fn normalise_king_ring(arr: &mut [Score; 24]) {
    make_nonincreasing(arr);
    shift_first_bucket_into_range(arr, -10, 20);
    limit_subsequent_drop(arr, 15, -200);
}

pub fn push_score_bounds(out: &mut Vec<ParamBounds>, b: ParamBounds) {
    out.push(b); // mg
    out.push(b); // eg
}

pub fn push_score_array_bounds<const N: usize>(out: &mut Vec<ParamBounds>, b: ParamBounds) {
    for _ in 0..N {
        push_score_bounds(out, b);
    }
}

pub fn push_pawn_pst_bounds(out: &mut Vec<ParamBounds>, b: ParamBounds) {
    for _ in 8..56 {
        push_score_bounds(out, b);
    }
}
