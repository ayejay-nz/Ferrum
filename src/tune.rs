use std::array;

use crate::evaluate::Score;

macro_rules! s {
    ($mg:literal, $eg:literal) => {
        Score { mg: $mg, eg: $eg }
    };
}

macro_rules! b {
    ($min:literal, $max:literal) => {
        ParamBounds {
            min: $min,
            max: $max,
        }
    };
}

type PST = [Score; 64];

pub const PIECE_VALUES: [Score; 5] = [
    s!(79, 77),
    s!(359, 349),
    s!(365, 340),
    s!(559, 519),
    s!(1099, 1099),
];

pub const DOUBLED_PAWNS: Score = s!(-5, -36);
pub const TRIPLED_PAWNS: Score = s!(-99, -54);
pub const QUADRUPLED_PAWNS: Score = s!(-171, -381);

pub const ISOLATED_PAWN: [Score; 4] = [s!(-15, 9), s!(-17, -11), s!(-28, -7), s!(-40, -19)];
pub const PASSED_PAWN: [Score; 6] = [
    s!(6, -4),
    s!(8, 10),
    s!(28, 18),
    s!(44, 49),
    s!(44, 81),
    s!(82, 132),
];

pub const KNIGHT_OUTPOST: Score = s!(27, 3);
pub const DEFENDED_KNIGHT_OUTPOST: Score = s!(40, 18);

pub const BISHOP_PAIR: Score = s!(60, 36);
pub const BISHOP_SAME_COLOUR_PAWNS: [Score; 9] = [
    s!(27, -8),
    s!(30, -21),
    s!(29, -19),
    s!(26, -28),
    s!(27, -40),
    s!(30, -43),
    s!(24, -27),
    s!(20, 0),
    s!(-10, 23),
];

pub const ROOK_OPEN_FILE: Score = s!(49, -3);
pub const ROOK_SEMI_OPEN_FILE: Score = s!(16, 10);

pub const KING_ON_SEMI_OPEN_FILE: Score = s!(-14, 21);
pub const KING_ON_OPEN_FILE: Score = s!(-61, -7);
pub const KING_PAWN_SHIELD_DISTANCE: [Score; 4] =
    [s!(15, -35), s!(5, -21), s!(12, -16), s!(-20, 10)];
pub const KING_SHIELD_MISSING_PAWN: Score = s!(-13, -21);
pub const ENEMY_PAWN_DISTANCE_FROM_BACKRANK: [Score; 4] =
    [s!(0, -7), s!(-19, -1), s!(-8, -1), s!(-1, -4)];

// King attack ring values
pub const KING_RING_PAWN_WEIGHT: Score = s!(1, 1);
pub const KING_RING_KNIGHT_WEIGHT: Score = s!(2, 2);
pub const KING_RING_BISHOP_WEIGHT: Score = s!(2, 2);
pub const KING_RING_ROOK_WEIGHT: Score = s!(3, 3);
pub const KING_RING_QUEEN_WEIGHT: Score = s!(5, 5);
pub const KING_RING_ATTACKS: [Score; 24] = [
    s!(5, 5),
    s!(3, 3),
    s!(1, 2),
    s!(1, 1),
    s!(0, 0),
    s!(0, 0),
    s!(0, 0),
    s!(0, 0),
    s!(-3, -3),
    s!(-3, -3),
    s!(-8, -8),
    s!(-8, -8),
    s!(-13, -13),
    s!(-13, -13),
    s!(-21, -21),
    s!(-21, -21),
    s!(-30, -30),
    s!(-30, -30),
    s!(-40, -40),
    s!(-40, -40),
    s!(-45, -45),
    s!(-45, -45),
    s!(-50, -50),
    s!(-50, -50),
];

// Adjustment values based on the number of pawns left
pub const KNIGHT_ADJ: [Score; 9] = [
    s!(-40, -40),
    s!(-18, -36),
    s!(-12, -22),
    s!(-12, 2),
    s!(-5, 12),
    s!(7, 12),
    s!(19, 14),
    s!(25, 30),
    s!(33, 32),
];
pub const ROOK_ADJ: [Score; 9] = [
    s!(43, 15),
    s!(37, 5),
    s!(20, 5),
    s!(16, 5),
    s!(-2, -1),
    s!(-13, -5),
    s!(-31, -9),
    s!(-33, -9),
    s!(-33, -11),
];

// Mobility scores based on the number of moves available to a piece
pub const KNIGHT_MOBILITY: [Score; 9] = [
    s!(-47, -1),
    s!(-2, -14),
    s!(-1, -53),
    s!(12, -54),
    s!(23, -44),
    s!(23, -32),
    s!(30, -29),
    s!(45, -61),
    s!(44, -51),
];
pub const BISHOP_MOBILITY: [Score; 14] = [
    s!(-25, -52),
    s!(-12, -36),
    s!(2, -37),
    s!(18, -48),
    s!(10, -13),
    s!(25, -14),
    s!(18, 12),
    s!(23, 3),
    s!(26, 9),
    s!(19, 17),
    s!(19, 12),
    s!(29, -7),
    s!(-9, -51),
    s!(-10, -1),
];
pub const ROOK_MOBILITY: [Score; 15] = [
    s!(-7, 23),
    s!(5, 29),
    s!(5, 10),
    s!(19, -18),
    s!(8, 15),
    s!(13, 29),
    s!(30, 22),
    s!(24, 24),
    s!(22, 27),
    s!(29, 30),
    s!(29, 32),
    s!(8, 49),
    s!(40, 47),
    s!(50, 43),
    s!(22, 42),
];
pub const QUEEN_MOBILITY: [Score; 28] = [
    s!(-21, -31),
    s!(-30, 31),
    s!(-26, 48),
    s!(-24, 0),
    s!(-14, 47),
    s!(-12, 6),
    s!(-10, 15),
    s!(-9, -7),
    s!(-5, -2),
    s!(4, -14),
    s!(-11, 24),
    s!(-10, 24),
    s!(8, 20),
    s!(12, 8),
    s!(8, 23),
    s!(12, 34),
    s!(4, 49),
    s!(-3, 45),
    s!(38, 6),
    s!(8, 44),
    s!(37, 17),
    s!(-31, 40),
    s!(42, 25),
    s!(29, 39),
    s!(-10, -5),
    s!(27, 41),
    s!(-8, 39),
    s!(11, 41),
];

// Piece square tables
#[rustfmt::skip]
pub const PAWN_PST: [Score; 64] = [
    s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), 
    s!(   2,  130), s!(  30,  164), s!(  71,   89), s!(  55,   72), s!(  30,   67), s!(  26,  115), s!(  51,   90), s!( 144,   35), 
    s!(  -5,   90), s!(  69,   86), s!(  13,  101), s!(   0,   55), s!(  32,   50), s!(  37,   55), s!( -11,   76), s!( -22,   90), 
    s!(  24,   52), s!(  17,   52), s!(  28,   23), s!(  44,   16), s!(  59,   18), s!(  61,    9), s!(  23,   36), s!( -28,   28), 
    s!(  -9,   33), s!(  11,   39), s!(  19,   15), s!(  41,   20), s!(  25,   36), s!(  18,   11), s!(  -5,   37), s!( -37,   16), 
    s!(  14,   24), s!(   9,   37), s!(  30,   28), s!(  19,   36), s!(  31,   34), s!(  21,   17), s!(  41,   31), s!(   2,   21), 
    s!(   6,   33), s!(  25,   48), s!(  14,   36), s!(  12,   82), s!(  13,   51), s!(  40,   45), s!(  62,   33), s!(   0,   18), 
    s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0)
];
#[rustfmt::skip]
pub const KNIGHT_PST: [Score; 64] = [
    s!(-137, -139), s!(-165, -185), s!(-118,  -41), s!(-111,  -68), s!(  97,   50), s!(-189,  168), s!( -55,   -9), s!(-110, -107), 
    s!(-145,   43), s!( -83,    4), s!(  -8,  -15), s!(  32,  -34), s!( 114, -171), s!( -69,   59), s!(  62,  -88), s!(  39, -104), 
    s!(  57,  -22), s!( -54,   64), s!(  37,   48), s!( -47,   87), s!(  -7,   51), s!(-136,   80), s!(  98,  -93), s!( -24,   23), 
    s!( -51,   30), s!(  31,  -33), s!(   1,   68), s!(  74,   81), s!(  35,   24), s!(  31,   27), s!(  -6,   -2), s!( 105,  -71), 
    s!(   1,   -1), s!( -33,   38), s!(  32,   53), s!(  27,   63), s!(  45,   44), s!(  45,   26), s!(  70,   74), s!(  16,   10), 
    s!(   1,  -11), s!(  24,   33), s!(  44,    5), s!(  52,   47), s!(  46,   29), s!(  47,  -13), s!(  57,  -70), s!(  44,  -73), 
    s!(   5,  -63), s!(  25,  -88), s!(  58,   19), s!(  45,  -17), s!(  55,  -21), s!(  54,   17), s!(  32,    5), s!(  47,   32), 
    s!(  98,  -37), s!(  25,  -33), s!( -64,   36), s!(  12,   11), s!(  56,  -46), s!(   6,   70), s!(  29,  -52), s!( 110,  -96)
];
#[rustfmt::skip]
pub const BISHOP_PST: [Score; 64] = [
    s!(  22,    3), s!(   8,  -10), s!( -72,   36), s!(  34,  -62), s!( -22,   -5), s!( -42,  -22), s!( -89,   70), s!( -72,   79), 
    s!( -70,   34), s!(  35,  -11), s!(  12,   52), s!( -85,   54), s!(  -5,   -4), s!(  36,  -14), s!(  23,  -31), s!(  33,    0), 
    s!(  21,    4), s!(  52,  -21), s!(  44,    0), s!( -57,   72), s!(  -8,   24), s!(  52,   60), s!(  34,   -1), s!(  44,  -15), 
    s!( -33,   60), s!(  27,   34), s!(  68,    6), s!(  48,   41), s!(  62,   23), s!(  -3,   62), s!(  31,   19), s!( -21,   39), 
    s!( -21,   15), s!(  52,  -16), s!(  35,    4), s!(  66,   28), s!(  23,   71), s!(  30,   31), s!(  56,  -26), s!( -16,  -34), 
    s!(  46,  -13), s!(  64,   -5), s!(  40,   35), s!(  33,   23), s!(  44,   17), s!(  81,  -17), s!(  44,  -59), s!(  15,   -8), 
    s!( -26,   30), s!(  58,    2), s!(  24,   33), s!(  44,   -2), s!(  53,   12), s!(  42,   14), s!(  87,   -1), s!(   8,   18), 
    s!(  10,   15), s!(  -4,   48), s!(  36,  -16), s!(   2,   11), s!(  29,   36), s!(   8,   39), s!( -49,   39), s!(  -7,   -9)
];
#[rustfmt::skip]
pub const ROOK_PST: [Score; 64] = [
    s!(  98,    4), s!( -17,   61), s!(  84,    4), s!(  72,    6), s!( -20,   32), s!( -89,   88), s!(  -9,    3), s!( -75,   52), 
    s!( -18,   35), s!(  -1,   44), s!( -36,   82), s!(   5,   55), s!(  69,   21), s!(  79,   -5), s!(  75,   59), s!( -40,   34), 
    s!( -40,   55), s!(  14,   27), s!( -67,   44), s!(   5,   45), s!(  58,   34), s!( -33,   33), s!(   0,   37), s!(  46,   13), 
    s!( -82,   48), s!( -99,   59), s!(  -8,   22), s!(  45,   31), s!( -46,   38), s!( -73,   59), s!( -58,   33), s!( -65,   27), 
    s!( -34,   29), s!( -38,   21), s!( -64,   26), s!( -66,   39), s!(  20,   41), s!(  26,   21), s!( -89,   48), s!(  52,  -15), 
    s!( -18,   -2), s!( -57,   10), s!(  37,   26), s!(  19,    3), s!(  22,    0), s!( -23,   31), s!( -57,   63), s!( -67,   50), 
    s!(  16,  -37), s!( -41,   47), s!(   6,   -1), s!(  17,   -6), s!(  29,   18), s!(  -3,   30), s!( -26,   69), s!( -94,   44), 
    s!(  11,   43), s!(  -1,   40), s!(  12,   35), s!(  15,   29), s!(  25,   30), s!(  31,   37), s!(  -4,   15), s!(  25,    1)
];
#[rustfmt::skip]
pub const QUEEN_PST: [Score; 64] = [
    s!(  -6,   22), s!(  32,  -53), s!(-104,  114), s!(-116,  138), s!(-179,   66), s!(  87,  -39), s!( 114, -180), s!( -75,  144), 
    s!(  28,  -55), s!(  11,  -51), s!(   7,   -6), s!(  28, -175), s!( -54,    6), s!(-111,   33), s!(  14,  131), s!(  62,  -68), 
    s!(  -4,   50), s!(  49,  -22), s!(  74,  -49), s!(  94,  -46), s!(  50,   75), s!( -82,  185), s!( 161,  -68), s!(   9,   80), 
    s!(  73, -181), s!(  37,   26), s!(  52,  -34), s!(  16,   78), s!(  -5,   93), s!( 137,  -36), s!( -28,  131), s!(  61,   20), 
    s!(  71, -120), s!(  48,  -18), s!(  63,  -59), s!(  34,  110), s!(  59,   37), s!(  70,   -9), s!(  77,   41), s!(  37,   23), 
    s!( 103, -118), s!(  82,  -49), s!(   5,   78), s!(  58,  -10), s!(  57,   41), s!(  76,  -23), s!(  84,  -77), s!(  30,  130), 
    s!( 110, -118), s!(  76,  125), s!(  95,  -76), s!( 100,  -64), s!( 104,  -48), s!(  83,   -1), s!( 162, -194), s!(  29,   36), 
    s!(  67,   -8), s!(  80,  -68), s!(  77,  -24), s!(  94,  -59), s!(  52,   37), s!(  17,   47), s!( 151, -117), s!(  -8,  160)
];
#[rustfmt::skip]
pub const KING_PST: [Score; 64] = [
    s!( -88,  -39), s!(  -9,  -74), s!(-103,    8), s!(-105,   78), s!(  45, -100), s!(-113,  -58), s!( -32, -112), s!( -23,  -15), 
    s!( -53,   28), s!( 172,  -30), s!(-152,  106), s!( -29,   45), s!( -50,   39), s!( 146,  -18), s!( 112,  -11), s!(-170,  -21), 
    s!( 176,  -90), s!( 195,  -35), s!( 101,   17), s!( -34,   33), s!(-110,   26), s!( -68,   67), s!(-137,   77), s!( 174,  -20), 
    s!(-110,   38), s!(  18,  -22), s!(-141,   66), s!( 145,    9), s!( 119,   36), s!(-199,   51), s!(-178,   60), s!( -81,   29), 
    s!(  76,  -59), s!(  48,    8), s!(  73,    9), s!(-128,   52), s!(  14,   31), s!(  71,   27), s!(  57,    7), s!( 172,  -29), 
    s!(-194,   33), s!( -39,    5), s!( 138,  -13), s!(-102,   49), s!(  36,   22), s!(  -5,   14), s!(  54,   -1), s!(  13,    9), 
    s!(-100,  -37), s!( 112,  -13), s!(  -8,   20), s!( -15,   29), s!( -35,   24), s!( -20,   30), s!(  45,   10), s!(  52,  -41), 
    s!( -85,    8), s!(  80,  -58), s!(  68,  -29), s!( -56,   13), s!(  26,  -14), s!( -16,    0), s!(  54,  -30), s!(  36,  -63)
];

fn push_score(out: &mut Vec<i32>, s: Score) {
    out.push(s.mg);
    out.push(s.eg);
}

fn push_score_array<const N: usize>(out: &mut Vec<i32>, arr: &[Score; N]) {
    for &s in arr {
        push_score(out, s);
    }
}

fn push_pawn_pst(out: &mut Vec<i32>, pst: &[Score; 64]) {
    for (i, &s) in pst.iter().enumerate() {
        // Skip 1st and 8th rank
        if !(8..56).contains(&i) {
            continue;
        }
        out.push(s.mg);
        out.push(s.eg);
    }
}

fn next_score<I: Iterator<Item = i32>>(it: &mut I) -> Score {
    Score {
        mg: it.next().unwrap(),
        eg: it.next().unwrap(),
    }
}

fn next_score_array<const N: usize, I: Iterator<Item = i32>>(it: &mut I) -> [Score; N] {
    array::from_fn(|_| next_score(it))
}

fn next_pawn_pst<I: Iterator<Item = i32>>(it: &mut I, base: &[Score; 64]) -> [Score; 64] {
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

fn make_nondecreasing<const N: usize>(arr: &mut [Score; N]) {
    for i in 1..N {
        arr[i].mg = arr[i].mg.max(arr[i - 1].mg);
        arr[i].eg = arr[i].eg.max(arr[i - 1].eg);
    }
}

fn make_nonincreasing<const N: usize>(arr: &mut [Score; N]) {
    for i in 1..N {
        arr[i].mg = arr[i].mg.min(arr[i - 1].mg);
        arr[i].eg = arr[i].eg.min(arr[i - 1].eg);
    }
}

fn normalise_mean_zero<const N: usize>(base: &mut Score, arr: &mut [Score; N]) {
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

fn normalise_king_ring(arr: &mut [Score; 24]) {
    make_nonincreasing(arr);
    shift_first_bucket_into_range(arr, -10, 20);
    limit_subsequent_drop(arr, 15, -200);
}

fn push_score_bounds(out: &mut Vec<ParamBounds>, b: ParamBounds) {
    out.push(b); // mg
    out.push(b); // eg
}

fn push_score_array_bounds<const N: usize>(out: &mut Vec<ParamBounds>, b: ParamBounds) {
    for _ in 0..N {
        push_score_bounds(out, b);
    }
}

fn push_pawn_pst_bounds(out: &mut Vec<ParamBounds>, b: ParamBounds) {
    for _ in 8..56 {
        push_score_bounds(out, b);
    }
}

pub trait TunableParams: Sync + Sized {
    fn pack(&self) -> Vec<i32>;
    fn unpack(values: &[i32]) -> Self;
    fn flat_bounds() -> Vec<ParamBounds>;
    fn project(&mut self);
    fn default() -> Self;

    fn clamp(&mut self) {
        let mut theta = self.pack();
        let bounds = Self::flat_bounds();

        for i in 0..theta.len() {
            theta[i] = theta[i].clamp(bounds[i].min, bounds[i].max);
        }

        *self = Self::unpack(&theta);
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ParamBounds {
    pub min: i32,
    pub max: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct Params {
    pub pawn_pst: PST,
    pub knight_pst: PST,
    pub bishop_pst: PST,
    pub rook_pst: PST,
    pub queen_pst: PST,
    pub king_pst: PST,

    pub pawn_value: Score,
    pub knight_value: Score,
    pub bishop_value: Score,
    pub rook_value: Score,
    pub queen_value: Score,

    pub knight_outpost: Score,
    pub defended_knight_outpost: Score,

    pub bishop_pair: Score,
    pub bishop_same_colour_pawns: [Score; 9],

    pub rook_open_file: Score,
    pub rook_semi_open_file: Score,

    pub doubled_pawns: Score,
    pub tripled_pawns: Score,
    pub quadrupled_pawns: Score,
    pub isolated_pawn: [Score; 4],
    pub passed_pawn: [Score; 6],

    pub king_on_open_file: Score,
    pub king_on_semi_open_file: Score,
    pub king_shield_missing_pawn: Score,
    pub king_pawn_shield_distance: [Score; 4],
    pub enemy_pawn_distance_from_backrank: [Score; 4],

    pub king_ring_pawn_weight: Score,
    pub king_ring_knight_weight: Score,
    pub king_ring_bishop_weight: Score,
    pub king_ring_rook_weight: Score,
    pub king_ring_queen_weight: Score,
    pub king_ring_attacks: [Score; 24],

    pub knight_adj: [Score; 9],
    pub rook_adj: [Score; 9],

    pub knight_mobility: [Score; 9],
    pub bishop_mobility: [Score; 14],
    pub rook_mobility: [Score; 15],
    pub queen_mobility: [Score; 28],
}

impl TunableParams for Params {
    fn pack(&self) -> Vec<i32> {
        let mut out = Vec::new();

        push_pawn_pst(&mut out, &self.pawn_pst);
        push_score_array(&mut out, &self.knight_pst);
        push_score_array(&mut out, &self.bishop_pst);
        push_score_array(&mut out, &self.rook_pst);
        push_score_array(&mut out, &self.queen_pst);
        push_score_array(&mut out, &self.king_pst);

        push_score(&mut out, self.pawn_value);
        push_score(&mut out, self.knight_value);
        push_score(&mut out, self.bishop_value);
        push_score(&mut out, self.rook_value);
        push_score(&mut out, self.queen_value);

        push_score(&mut out, self.knight_outpost);
        push_score(&mut out, self.defended_knight_outpost);

        push_score(&mut out, self.bishop_pair);
        push_score_array(&mut out, &self.bishop_same_colour_pawns);

        push_score(&mut out, self.rook_open_file);
        push_score(&mut out, self.rook_semi_open_file);

        push_score(&mut out, self.doubled_pawns);
        push_score(&mut out, self.tripled_pawns);
        push_score(&mut out, self.quadrupled_pawns);
        push_score_array(&mut out, &self.isolated_pawn);
        push_score_array(&mut out, &self.passed_pawn);

        push_score(&mut out, self.king_on_open_file);
        push_score(&mut out, self.king_on_semi_open_file);
        push_score(&mut out, self.king_shield_missing_pawn);
        push_score_array(&mut out, &self.king_pawn_shield_distance);
        push_score_array(&mut out, &self.enemy_pawn_distance_from_backrank);

        push_score(&mut out, self.king_ring_pawn_weight);
        push_score(&mut out, self.king_ring_knight_weight);
        push_score(&mut out, self.king_ring_bishop_weight);
        push_score(&mut out, self.king_ring_rook_weight);
        push_score(&mut out, self.king_ring_queen_weight);
        push_score_array(&mut out, &self.king_ring_attacks);

        push_score_array(&mut out, &self.knight_adj);
        push_score_array(&mut out, &self.rook_adj);

        push_score_array(&mut out, &self.knight_mobility);
        push_score_array(&mut out, &self.bishop_mobility);
        push_score_array(&mut out, &self.rook_mobility);
        push_score_array(&mut out, &self.queen_mobility);

        out
    }

    fn unpack(values: &[i32]) -> Self {
        let mut it = values.iter().copied();

        let params = Self {
            pawn_pst: next_pawn_pst(&mut it, &PAWN_PST),
            knight_pst: next_score_array(&mut it),
            bishop_pst: next_score_array(&mut it),
            rook_pst: next_score_array(&mut it),
            queen_pst: next_score_array(&mut it),
            king_pst: next_score_array(&mut it),

            pawn_value: next_score(&mut it),
            knight_value: next_score(&mut it),
            bishop_value: next_score(&mut it),
            rook_value: next_score(&mut it),
            queen_value: next_score(&mut it),

            knight_outpost: next_score(&mut it),
            defended_knight_outpost: next_score(&mut it),

            bishop_pair: next_score(&mut it),
            bishop_same_colour_pawns: next_score_array(&mut it),

            rook_open_file: next_score(&mut it),
            rook_semi_open_file: next_score(&mut it),

            doubled_pawns: next_score(&mut it),
            tripled_pawns: next_score(&mut it),
            quadrupled_pawns: next_score(&mut it),
            isolated_pawn: next_score_array(&mut it),
            passed_pawn: next_score_array(&mut it),

            king_on_open_file: next_score(&mut it),
            king_on_semi_open_file: next_score(&mut it),
            king_shield_missing_pawn: next_score(&mut it),
            king_pawn_shield_distance: next_score_array(&mut it),
            enemy_pawn_distance_from_backrank: next_score_array(&mut it),

            king_ring_pawn_weight: next_score(&mut it),
            king_ring_knight_weight: next_score(&mut it),
            king_ring_bishop_weight: next_score(&mut it),
            king_ring_rook_weight: next_score(&mut it),
            king_ring_queen_weight: next_score(&mut it),
            king_ring_attacks: next_score_array(&mut it),

            knight_adj: next_score_array(&mut it),
            rook_adj: next_score_array(&mut it),

            knight_mobility: next_score_array(&mut it),
            bishop_mobility: next_score_array(&mut it),
            rook_mobility: next_score_array(&mut it),
            queen_mobility: next_score_array(&mut it),
        };

        debug_assert!(it.next().is_none());
        params
    }

    fn flat_bounds() -> Vec<ParamBounds> {
        let mut out = Vec::new();

        let [
            pawn_pst,
            knight_pst,
            bishop_pst,
            rook_pst,
            queen_pst,
            king_pst,
            pawn_value,
            knight_value,
            bishop_value,
            rook_value,
            queen_value,
            knight_outpost,
            defended_knight_outpost,
            bishop_pair,
            bishop_same_colour_pawns,
            rook_open_file,
            rook_semi_open_file,
            doubled_pawns,
            tripled_pawns,
            quadrupled_pawns,
            isolated_pawn,
            passed_pawn,
            king_on_open_file,
            king_on_semi_open_file,
            king_shield_missing_pawn,
            king_pawn_shield_distance,
            enemy_pawn_distance_from_backrank,
            king_ring_pawn_weight,
            king_ring_knight_weight,
            king_ring_bishop_weight,
            king_ring_rook_weight,
            king_ring_queen_weight,
            king_ring_attacks,
            knight_adj,
            rook_adj,
            knight_mobility,
            bishop_mobility,
            rook_mobility,
            queen_mobility,
        ] = PARAM_BOUNDS;

        push_pawn_pst_bounds(&mut out, pawn_pst);
        push_score_array_bounds::<64>(&mut out, knight_pst);
        push_score_array_bounds::<64>(&mut out, bishop_pst);
        push_score_array_bounds::<64>(&mut out, rook_pst);
        push_score_array_bounds::<64>(&mut out, queen_pst);
        push_score_array_bounds::<64>(&mut out, king_pst);

        push_score_bounds(&mut out, pawn_value);
        push_score_bounds(&mut out, knight_value);
        push_score_bounds(&mut out, bishop_value);
        push_score_bounds(&mut out, rook_value);
        push_score_bounds(&mut out, queen_value);

        push_score_bounds(&mut out, knight_outpost);
        push_score_bounds(&mut out, defended_knight_outpost);

        push_score_bounds(&mut out, bishop_pair);
        push_score_array_bounds::<9>(&mut out, bishop_same_colour_pawns);

        push_score_bounds(&mut out, rook_open_file);
        push_score_bounds(&mut out, rook_semi_open_file);

        push_score_bounds(&mut out, doubled_pawns);
        push_score_bounds(&mut out, tripled_pawns);
        push_score_bounds(&mut out, quadrupled_pawns);
        push_score_array_bounds::<4>(&mut out, isolated_pawn);
        push_score_array_bounds::<6>(&mut out, passed_pawn);

        push_score_bounds(&mut out, king_on_open_file);
        push_score_bounds(&mut out, king_on_semi_open_file);
        push_score_bounds(&mut out, king_shield_missing_pawn);
        push_score_array_bounds::<4>(&mut out, king_pawn_shield_distance);
        push_score_array_bounds::<4>(&mut out, enemy_pawn_distance_from_backrank);

        push_score_bounds(&mut out, king_ring_pawn_weight);
        push_score_bounds(&mut out, king_ring_knight_weight);
        push_score_bounds(&mut out, king_ring_bishop_weight);
        push_score_bounds(&mut out, king_ring_rook_weight);
        push_score_bounds(&mut out, king_ring_queen_weight);
        push_score_array_bounds::<24>(&mut out, king_ring_attacks);

        push_score_array_bounds::<9>(&mut out, knight_adj);
        push_score_array_bounds::<9>(&mut out, rook_adj);

        push_score_array_bounds::<9>(&mut out, knight_mobility);
        push_score_array_bounds::<14>(&mut out, bishop_mobility);
        push_score_array_bounds::<15>(&mut out, rook_mobility);
        push_score_array_bounds::<28>(&mut out, queen_mobility);

        out
    }

    fn project(&mut self) {
        make_nonincreasing(&mut self.king_ring_attacks);
        make_nondecreasing(&mut self.passed_pawn);

        make_nondecreasing(&mut self.knight_adj);
        make_nonincreasing(&mut self.rook_adj);

        self.tripled_pawns.mg = self.tripled_pawns.mg.min(self.doubled_pawns.mg);
        self.tripled_pawns.eg = self.tripled_pawns.eg.min(self.doubled_pawns.eg);

        self.quadrupled_pawns.mg = self.quadrupled_pawns.mg.min(self.tripled_pawns.mg);
        self.quadrupled_pawns.eg = self.quadrupled_pawns.eg.min(self.tripled_pawns.eg);

        // Material value
        self.bishop_value.mg = self.bishop_value.mg.max(self.knight_value.mg - 20);
        self.bishop_value.eg = self.bishop_value.eg.max(self.knight_value.eg - 20);

        self.rook_value.mg = self.rook_value.mg.max(self.bishop_value.mg + 100);
        self.rook_value.eg = self.rook_value.eg.max(self.bishop_value.eg + 100);

        // Normalise knight/rook adjustment tables, bishop same colour pawns, and all mobility scores
        // Results in piece values more accurately representing their true value
        normalise_mean_zero(&mut self.knight_value, &mut self.knight_adj);
        normalise_mean_zero(&mut self.rook_value, &mut self.rook_adj);

        normalise_mean_zero(&mut self.knight_value, &mut self.knight_mobility);
        normalise_mean_zero(&mut self.bishop_value, &mut self.bishop_mobility);
        normalise_mean_zero(&mut self.rook_value, &mut self.rook_mobility);
        normalise_mean_zero(&mut self.queen_value, &mut self.queen_mobility);

        normalise_mean_zero(&mut self.bishop_value, &mut self.bishop_same_colour_pawns);

        self.clamp();

        normalise_king_ring(&mut self.king_ring_attacks);
    }

    fn default() -> Params {
        DEFAULT_PARAMS
    }
}

pub const DEFAULT_PARAMS: Params = Params {
    pawn_pst: PAWN_PST,
    knight_pst: KNIGHT_PST,
    bishop_pst: BISHOP_PST,
    rook_pst: ROOK_PST,
    queen_pst: QUEEN_PST,
    king_pst: KING_PST,

    pawn_value: PIECE_VALUES[0],
    knight_value: PIECE_VALUES[1],
    bishop_value: PIECE_VALUES[2],
    rook_value: PIECE_VALUES[3],
    queen_value: PIECE_VALUES[4],

    knight_outpost: KNIGHT_OUTPOST,
    defended_knight_outpost: DEFENDED_KNIGHT_OUTPOST,

    bishop_pair: BISHOP_PAIR,
    bishop_same_colour_pawns: BISHOP_SAME_COLOUR_PAWNS,

    rook_open_file: ROOK_OPEN_FILE,
    rook_semi_open_file: ROOK_SEMI_OPEN_FILE,

    doubled_pawns: DOUBLED_PAWNS,
    tripled_pawns: TRIPLED_PAWNS,
    quadrupled_pawns: QUADRUPLED_PAWNS,
    isolated_pawn: ISOLATED_PAWN,
    passed_pawn: PASSED_PAWN,

    king_on_open_file: KING_ON_OPEN_FILE,
    king_on_semi_open_file: KING_ON_SEMI_OPEN_FILE,
    king_shield_missing_pawn: KING_SHIELD_MISSING_PAWN,
    king_pawn_shield_distance: KING_PAWN_SHIELD_DISTANCE,
    enemy_pawn_distance_from_backrank: ENEMY_PAWN_DISTANCE_FROM_BACKRANK,

    king_ring_pawn_weight: KING_RING_PAWN_WEIGHT,
    king_ring_knight_weight: KING_RING_KNIGHT_WEIGHT,
    king_ring_bishop_weight: KING_RING_BISHOP_WEIGHT,
    king_ring_rook_weight: KING_RING_ROOK_WEIGHT,
    king_ring_queen_weight: KING_RING_QUEEN_WEIGHT,
    king_ring_attacks: KING_RING_ATTACKS,

    knight_adj: KNIGHT_ADJ,
    rook_adj: ROOK_ADJ,

    knight_mobility: KNIGHT_MOBILITY,
    bishop_mobility: BISHOP_MOBILITY,
    rook_mobility: ROOK_MOBILITY,
    queen_mobility: QUEEN_MOBILITY,
};

#[rustfmt::skip]
pub const PARAM_BOUNDS: [ParamBounds; 39] = [
    b!(-50, 200),  // pawn pst
    b!(-200, 200), // knight pst
    b!(-100, 100), // bishop pst
    b!(-100, 100), // rook pst
    b!(-200, 200), // queen pst
    b!(-200, 200), // king pst

    b!(70, 100),   // pawn value
    b!(240, 360),  // knight value
    b!(250, 370),  // bishop value
    b!(400, 560),  // rook value
    b!(850, 1100), // queen value

    b!(-5, 30),    // knight outpost
    b!(-5, 60),    // defended knight outpost

    b!(-5, 100),   // bishop pair
    b!(-50, 30),   // bishop same colour pawns

    b!(-5, 100),   // rook open file
    b!(-5, 100),   // rook semi-open file

    b!(-100, -1),  // doubled pawns
    b!(-200, -1),  // tripled pawns
    b!(-400, -50), // quadrupled pawns
    b!(-50, 10),   // isolated pawn
    b!(-5, 200),   // passed pawn

    b!(-75, 5),    // king on open file
    b!(-40, 40),   // king on semi open file
    b!(-75, 0),    // king shield missing pawn
    b!(-40, 40),   // king pawn shield distance
    b!(-50, 0),    // enemy pawn distance from backrank

    b!(1, 2),      // king ring pawn weights
    b!(2, 3),      // king ring knight weights
    b!(2, 3),      // king ring bishop weights
    b!(3, 4),      // king ring rook weights
    b!(4, 5),      // king ring queen weights
    b!(-200, 20),  // king ring attacks

    b!(-60, 60),   // knight adj
    b!(-60, 60),   // rook adj

    b!(-75, 75),   // knight mobility
    b!(-75, 75),   // bishop mobility
    b!(-50, 50),   // rook mobility
    b!(-50, 50),   // queen mobility
];

pub const LAZY_PIECE_VALUES: [Score; 5] = [
    s!(82, 94),
    s!(337, 281),
    s!(365, 297),
    s!(477, 512),
    s!(1025, 936),
];

// Lazy PSTs
#[rustfmt::skip]
const LAZY_PAWN_PST: PST = [
    s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0),
    s!(  98,  178), s!( 134,  173), s!(  61,  158), s!(  95,  134), s!(  68,  147), s!( 126,  132), s!(  34,  165), s!( -11,  187),
    s!(  -6,   94), s!(   7,  100), s!(  26,   85), s!(  31,   67), s!(  65,   56), s!(  56,   53), s!(  25,   82), s!( -20,   84),
    s!( -14,   32), s!(  13,   24), s!(   6,   13), s!(  21,    5), s!(  23,   -2), s!(  12,    4), s!(  17,   17), s!( -23,   17),
    s!( -27,   13), s!(  -2,    9), s!(  -5,   -3), s!(  12,   -7), s!(  17,   -7), s!(   6,   -8), s!(  10,    3), s!( -25,   -1),
    s!( -26,    4), s!(  -4,    7), s!(  -4,   -6), s!( -10,    1), s!(   3,    0), s!(   3,   -5), s!(  33,   -1), s!( -12,   -8),
    s!( -35,   13), s!(  -1,    8), s!( -20,    8), s!( -23,   10), s!( -15,   13), s!(  24,    0), s!(  38,    2), s!( -22,   -7),
    s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0), s!(   0,    0),
];
#[rustfmt::skip]
const LAZY_KNIGHT_PST: PST = [
    s!(-167,  -58), s!( -89,  -38), s!( -34,  -13), s!( -49,  -28), s!(  61,  -31), s!( -97,  -27), s!( -15,  -63), s!(-107,  -99),
    s!( -73,  -25), s!( -41,   -8), s!(  72,  -25), s!(  36,   -2), s!(  23,   -9), s!(  62,  -25), s!(   7,  -24), s!( -17,  -52),
    s!( -47,  -24), s!(  60,  -20), s!(  37,   10), s!(  65,    9), s!(  84,   -1), s!( 129,   -9), s!(  73,  -19), s!(  44,  -41),
    s!(  -9,  -17), s!(  17,    3), s!(  19,   22), s!(  53,   22), s!(  37,   22), s!(  69,   11), s!(  18,    8), s!(  22,  -18),
    s!( -13,  -18), s!(   4,   -6), s!(  16,   16), s!(  13,   25), s!(  28,   16), s!(  19,   17), s!(  21,    4), s!(  -8,  -18),
    s!( -23,  -23), s!(  -9,   -3), s!(  12,   -1), s!(  10,   15), s!(  19,   10), s!(  17,   -3), s!(  25,  -20), s!( -16,  -22),
    s!( -29,  -42), s!( -53,  -20), s!( -12,  -10), s!(  -3,   -5), s!(  -1,   -2), s!(  18,  -20), s!( -14,  -23), s!( -19,  -44),
    s!(-105,  -29), s!( -21,  -51), s!( -58,  -23), s!( -33,  -15), s!( -17,  -22), s!( -28,  -18), s!( -19,  -50), s!( -23,  -64)
];
#[rustfmt::skip]
const LAZY_BISHOP_PST: PST = [
    s!( -29,  -14), s!(   4,  -21), s!( -82,  -11), s!( -37,   -8), s!( -25,   -7), s!( -42,   -9), s!(   7,  -17), s!(  -8,  -24),
    s!( -26,   -8), s!(  16,   -4), s!( -18,    7), s!( -13,  -12), s!(  30,   -3), s!(  59,  -13), s!(  18,   -4), s!( -47,  -14),
    s!( -16,    2), s!(  37,   -8), s!(  43,    0), s!(  40,   -1), s!(  35,   -2), s!(  50,    6), s!(  37,    0), s!(  -2,    4),
    s!(  -4,   -3), s!(   5,    9), s!(  19,   12), s!(  50,    9), s!(  37,   14), s!(  37,   10), s!(   7,    3), s!(  -2,    2),
    s!(  -6,   -6), s!(  13,    3), s!(  13,   13), s!(  26,   19), s!(  34,    7), s!(  12,   10), s!(  10,   -3), s!(   4,   -9),
    s!(   0,  -12), s!(  15,   -3), s!(  15,    8), s!(  15,   10), s!(  14,   13), s!(  27,    3), s!(  18,   -7), s!(  10,  -15),
    s!(   4,  -14), s!(  15,  -18), s!(  16,   -7), s!(   0,   -1), s!(   7,    4), s!(  21,   -9), s!(  33,  -15), s!(   1,  -27),
    s!( -33,  -23), s!(  -3,   -9), s!( -14,  -23), s!( -21,   -5), s!( -13,   -9), s!( -12,  -16), s!( -39,   -5), s!( -21,  -17)
];
#[rustfmt::skip]
const LAZY_ROOK_PST: PST = [
    s!(  32,   13), s!(  42,   10), s!(  32,   18), s!(  51,   15), s!(  63,   12), s!(   9,   12), s!(  31,    8), s!(  43,    5),
    s!(  27,   11), s!(  32,   13), s!(  58,   13), s!(  62,   11), s!(  80,   -3), s!(  67,    3), s!(  26,    8), s!(  44,    3),
    s!(  -5,    7), s!(  19,    7), s!(  26,    7), s!(  36,    5), s!(  17,    4), s!(  45,   -3), s!(  61,   -5), s!(  16,   -3),
    s!( -24,    4), s!( -11,    3), s!(   7,   13), s!(  26,    1), s!(  24,    2), s!(  35,    1), s!(  -8,   -1), s!( -20,    2),
    s!( -36,    3), s!( -26,    5), s!( -12,    8), s!(  -1,    4), s!(   9,   -5), s!(  -7,   -6), s!(   6,   -8), s!( -23,  -11),
    s!( -45,   -4), s!( -25,    0), s!( -16,   -5), s!( -17,   -1), s!(   3,   -7), s!(   0,  -12), s!(  -5,   -8), s!( -33,  -16),
    s!( -44,   -6), s!( -16,   -6), s!( -20,    0), s!(  -9,    2), s!(  -1,   -9), s!(  11,   -9), s!(  -6,  -11), s!( -71,   -3),
    s!( -19,   -9), s!( -13,    2), s!(   1,    3), s!(  17,   -1), s!(  16,   -5), s!(   7,  -13), s!( -37,    4), s!( -26,  -20)
];
#[rustfmt::skip]
const LAZY_QUEEN_PST: PST = [
    s!( -28,   -9), s!(   0,   22), s!(  29,   22), s!(  12,   27), s!(  59,   27), s!(  44,   19), s!(  43,   10), s!(  45,   20),
    s!( -24,  -17), s!( -39,   20), s!(  -5,   32), s!(   1,   41), s!( -16,   58), s!(  57,   25), s!(  28,   30), s!(  54,    0),
    s!( -13,  -20), s!( -17,    6), s!(   7,    9), s!(   8,   49), s!(  29,   47), s!(  56,   35), s!(  47,   19), s!(  57,    9),
    s!( -27,    3), s!( -27,   22), s!( -16,   24), s!( -16,   45), s!(  -1,   57), s!(  17,   40), s!(  -2,   57), s!(   1,   36),
    s!(  -9,  -18), s!( -26,   28), s!(  -9,   19), s!( -10,   47), s!(  -2,   31), s!(  -4,   34), s!(   3,   39), s!(  -3,   23),
    s!( -14,  -16), s!(   2,  -27), s!( -11,   15), s!(  -2,    6), s!(  -5,    9), s!(   2,   17), s!(  14,   10), s!(   5,    5),
    s!( -35,  -22), s!(  -8,  -23), s!(  11,  -30), s!(   2,  -16), s!(   8,  -16), s!(  15,  -23), s!(  -3,  -36), s!(   1,  -32),
    s!(  -1,  -33), s!( -18,  -28), s!(  -9,  -22), s!(  10,  -43), s!( -15,   -5), s!( -25,  -32), s!( -31,  -20), s!( -50,  -41)
];
#[rustfmt::skip]
const LAZY_KING_PST: PST = [
    s!( -65,  -74), s!(  23,  -35), s!(  16,  -18), s!( -15,  -18), s!( -56,  -11), s!( -34,   15), s!(   2,    4), s!(  13,  -17),
    s!(  29,  -12), s!(  -1,   17), s!( -20,   14), s!(  -7,   17), s!(  -8,   17), s!(  -4,   38), s!( -38,   23), s!( -29,   11),
    s!(  -9,   10), s!(  24,   17), s!(   2,   23), s!( -16,   15), s!( -20,   20), s!(   6,   45), s!(  22,   44), s!( -22,   13),
    s!( -17,   -8), s!( -20,   22), s!( -12,   24), s!( -27,   27), s!( -30,   26), s!( -25,   33), s!( -14,   26), s!( -36,    3),
    s!( -49,  -18), s!(  -1,   -4), s!( -27,   21), s!( -39,   24), s!( -46,   27), s!( -44,   23), s!( -33,    9), s!( -51,  -11),
    s!( -14,  -19), s!( -14,   -3), s!( -22,   11), s!( -46,   21), s!( -44,   23), s!( -30,   16), s!( -15,    7), s!( -27,   -9),
    s!(   1,  -27), s!(   7,  -11), s!(  -8,    4), s!( -64,   13), s!( -43,   14), s!( -16,    4), s!(   9,   -5), s!(   8,  -17),
    s!( -15,  -53), s!(  36,  -34), s!(  12,  -21), s!( -54,  -11), s!(   8,  -28), s!( -28,  -14), s!(  24,  -24), s!(  14,  -43)
];

#[derive(Copy, Clone, Debug)]
pub struct LazyParams {
    pub pawn_value: Score,
    pub knight_value: Score,
    pub bishop_value: Score,
    pub rook_value: Score,
    pub queen_value: Score,

    pub pawn_pst: PST,
    pub knight_pst: PST,
    pub bishop_pst: PST,
    pub rook_pst: PST,
    pub queen_pst: PST,
    pub king_pst: PST,
}

impl TunableParams for LazyParams {
    fn pack(&self) -> Vec<i32> {
        let mut out = Vec::new();

        push_pawn_pst(&mut out, &self.pawn_pst);
        push_score_array(&mut out, &self.knight_pst);
        push_score_array(&mut out, &self.bishop_pst);
        push_score_array(&mut out, &self.rook_pst);
        push_score_array(&mut out, &self.queen_pst);
        push_score_array(&mut out, &self.king_pst);

        push_score(&mut out, self.pawn_value);
        push_score(&mut out, self.knight_value);
        push_score(&mut out, self.bishop_value);
        push_score(&mut out, self.rook_value);
        push_score(&mut out, self.queen_value);

        out
    }

    fn unpack(values: &[i32]) -> Self {
        let mut it = values.iter().copied();

        let params = Self {
            pawn_pst: next_pawn_pst(&mut it, &LAZY_PAWN_PST),
            knight_pst: next_score_array(&mut it),
            bishop_pst: next_score_array(&mut it),
            rook_pst: next_score_array(&mut it),
            queen_pst: next_score_array(&mut it),
            king_pst: next_score_array(&mut it),

            pawn_value: next_score(&mut it),
            knight_value: next_score(&mut it),
            bishop_value: next_score(&mut it),
            rook_value: next_score(&mut it),
            queen_value: next_score(&mut it),
        };

        debug_assert!(it.next().is_none());
        params
    }

    fn flat_bounds() -> Vec<ParamBounds> {
        let mut out = Vec::new();

        let [
            pawn_pst,
            knight_pst,
            bishop_pst,
            rook_pst,
            queen_pst,
            king_pst,
            pawn_value,
            knight_value,
            bishop_value,
            rook_value,
            queen_value,
        ] = LAZY_PARAM_BOUNDS;

        push_pawn_pst_bounds(&mut out, pawn_pst);
        push_score_array_bounds::<64>(&mut out, knight_pst);
        push_score_array_bounds::<64>(&mut out, bishop_pst);
        push_score_array_bounds::<64>(&mut out, rook_pst);
        push_score_array_bounds::<64>(&mut out, queen_pst);
        push_score_array_bounds::<64>(&mut out, king_pst);

        push_score_bounds(&mut out, pawn_value);
        push_score_bounds(&mut out, knight_value);
        push_score_bounds(&mut out, bishop_value);
        push_score_bounds(&mut out, rook_value);
        push_score_bounds(&mut out, queen_value);

        out
    }

    fn project(&mut self) {
        // Material value
        self.bishop_value.mg = self.bishop_value.mg.max(self.knight_value.mg - 20);
        self.bishop_value.eg = self.bishop_value.eg.max(self.knight_value.eg - 20);

        self.rook_value.mg = self.rook_value.mg.max(self.bishop_value.mg + 100);
        self.rook_value.eg = self.rook_value.eg.max(self.bishop_value.eg + 100);

        self.clamp();
    }

    fn default() -> Self {
        DEFAULT_LAZY_PARAMS
    }
}

pub const DEFAULT_LAZY_PARAMS: LazyParams = LazyParams {
    pawn_value: LAZY_PIECE_VALUES[0],
    knight_value: LAZY_PIECE_VALUES[1],
    bishop_value: LAZY_PIECE_VALUES[2],
    rook_value: LAZY_PIECE_VALUES[3],
    queen_value: LAZY_PIECE_VALUES[4],

    pawn_pst: LAZY_PAWN_PST,
    knight_pst: LAZY_KNIGHT_PST,
    bishop_pst: LAZY_BISHOP_PST,
    rook_pst: LAZY_ROOK_PST,
    queen_pst: LAZY_QUEEN_PST,
    king_pst: LAZY_KING_PST,
};

#[rustfmt::skip]
pub const LAZY_PARAM_BOUNDS: [ParamBounds; 11] = [
    b!(-50, 200),  // pawn pst
    b!(-200, 200), // knight pst
    b!(-100, 100), // bishop pst
    b!(-100, 100), // rook pst
    b!(-200, 200), // queen pst
    b!(-200, 200), // king pst

    b!(70, 100),   // pawn value
    b!(240, 360),  // knight value
    b!(250, 370),  // bishop value
    b!(400, 560),  // rook value
    b!(850, 1100), // queen value
];
