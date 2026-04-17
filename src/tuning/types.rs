use std::fmt;

use crate::{
    params::{
        DEFAULT_LAZY_PARAMS, DEFAULT_PARAMS, LAZY_PARAM_COUNT, LAZY_PAWN_PST, LazyParams,
        PARAM_COUNT, PAWN_PST, Params,
    },
    position::Position,
    tuning::helpers::*,
};

macro_rules! b {
    ($min:literal, $max:literal) => {
        ParamBounds {
            min: $min,
            max: $max,
        }
    };
}

macro_rules! m {
    ($bounds:expr, $active:expr) => {
        ParamMeta {
            bounds: $bounds,
            active: $active,
        }
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameResult {
    BlackWin,
    Draw,
    WhiteWin,
}

impl GameResult {
    pub fn from_pgn_tag(tag: &str) -> Option<Self> {
        match tag {
            "0-1" => Some(Self::BlackWin),
            "1-0" => Some(Self::WhiteWin),
            "1/2-1/2" => Some(Self::Draw),
            "*" => None,
            _ => None,
        }
    }

    pub fn to_value(self) -> f64 {
        match self {
            Self::BlackWin => 0f64,
            Self::WhiteWin => 1f64,
            Self::Draw => 0.5,
        }
    }
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlackWin => f.write_str("0-1"),
            Self::WhiteWin => f.write_str("1-0"),
            Self::Draw => f.write_str("1/2-1/2"),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Sample {
    pub pos: Position,
    pub result: GameResult,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ParamBounds {
    pub min: i32,
    pub max: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct ParamMeta {
    pub bounds: ParamBounds,
    pub active: bool,
}

pub trait TuningConfig {
    type ParamType: Sync + Copy;

    fn pack(&self, params: &Self::ParamType) -> Vec<i32>;
    fn unpack(values: &[i32]) -> Self::ParamType;

    fn flat_param_meta(&self) -> Vec<ParamMeta>;
    fn project(&self, params: &mut Self::ParamType);
    fn default_params(&self) -> Self::ParamType;
    fn clamp(&self, params: &mut Self::ParamType);
}

#[derive(Clone, Copy, Debug)]
pub struct FullTuningConfig {
    pub params: Params,
    pub meta: [ParamMeta; PARAM_COUNT],
}

#[derive(Clone, Copy, Debug)]
pub struct LazyTuningConfig {
    pub params: LazyParams,
    pub meta: [ParamMeta; LAZY_PARAM_COUNT],
}

impl Default for FullTuningConfig {
    fn default() -> Self {
        Self {
            params: DEFAULT_PARAMS,
            meta: DEFAULT_PARAM_META,
        }
    }
}

impl Default for LazyTuningConfig {
    fn default() -> Self {
        Self {
            params: DEFAULT_LAZY_PARAMS,
            meta: DEFAULT_LAZY_PARAM_META,
        }
    }
}

impl TuningConfig for FullTuningConfig {
    type ParamType = Params;

    fn pack(&self, params: &Self::ParamType) -> Vec<i32> {
        let mut out = Vec::new();

        push_pawn_pst(&mut out, &params.pawn_pst);
        push_score_array(&mut out, &params.knight_pst);
        push_score_array(&mut out, &params.bishop_pst);
        push_score_array(&mut out, &params.rook_pst);
        push_score_array(&mut out, &params.queen_pst);
        push_score_array(&mut out, &params.king_pst);

        push_score(&mut out, params.pawn_value);
        push_score(&mut out, params.knight_value);
        push_score(&mut out, params.bishop_value);
        push_score(&mut out, params.rook_value);
        push_score(&mut out, params.queen_value);

        push_score(&mut out, params.knight_outpost);
        push_score(&mut out, params.defended_knight_outpost);

        push_score(&mut out, params.bishop_pair);
        push_score_array(&mut out, &params.bishop_same_colour_pawns);
        push_score(&mut out, params.fianchetto);

        push_score(&mut out, params.rook_open_file);
        push_score(&mut out, params.rook_semi_open_file);

        push_score(&mut out, params.doubled_pawns);
        push_score(&mut out, params.tripled_pawns);
        push_score(&mut out, params.quadrupled_pawns);
        push_score_array(&mut out, &params.isolated_pawn);
        push_score_array(&mut out, &params.passed_pawn);

        push_score(&mut out, params.king_on_open_file);
        push_score(&mut out, params.king_on_semi_open_file);
        push_score(&mut out, params.king_shield_missing_pawn);
        push_score_array(&mut out, &params.king_pawn_shield_distance);
        push_score_array(&mut out, &params.enemy_pawn_distance_from_backrank);

        push_score(&mut out, params.king_ring_pawn_weight);
        push_score(&mut out, params.king_ring_knight_weight);
        push_score(&mut out, params.king_ring_bishop_weight);
        push_score(&mut out, params.king_ring_rook_weight);
        push_score(&mut out, params.king_ring_queen_weight);
        push_score_array(&mut out, &params.king_ring_attacks);

        push_score_array(&mut out, &params.knight_adj);
        push_score_array(&mut out, &params.rook_adj);

        push_score_array(&mut out, &params.knight_mobility);
        push_score_array(&mut out, &params.bishop_mobility);
        push_score_array(&mut out, &params.rook_mobility);
        push_score_array(&mut out, &params.queen_mobility);

        out
    }

    fn unpack(values: &[i32]) -> Self::ParamType {
        let mut it = values.iter().copied();

        let params = Params {
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
            fianchetto: next_score(&mut it),

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

    fn flat_param_meta(&self) -> Vec<ParamMeta> {
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
            fianchetto,
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
        ] = self.meta;

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
        push_score_bounds(&mut out, fianchetto);

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

    /// Enforce that some parameter arrays are either monotone increasing or decreasing.
    /// Enforce that material value doesn't drift too far apart.
    /// Normalise knight/rook adjustment tables, bishop same colour pawns, and all mobility scores
    /// Results in piece values more accurately representing their true value.
    /// Normalise king ring attack values by ensuring they are logical and monotone decreasing.
    fn project(&self, params: &mut Self::ParamType) {
        if self.meta[22].active {
            make_nondecreasing(&mut params.passed_pawn);
        }
        if self.meta[33].active {
            make_nonincreasing(&mut params.king_ring_attacks);
        }

        if self.meta[34].active {
            make_nondecreasing(&mut params.knight_adj);

            if self.meta[7].active {
                normalise_mean_zero(&mut params.knight_value, &mut params.knight_adj);
            }
        }
        if self.meta[35].active {
            make_nonincreasing(&mut params.rook_adj);

            if self.meta[9].active {
                normalise_mean_zero(&mut params.rook_value, &mut params.rook_adj);
            }
        }

        if self.meta[19].active {
            params.tripled_pawns.mg = params.tripled_pawns.mg.min(params.doubled_pawns.mg);
            params.tripled_pawns.eg = params.tripled_pawns.eg.min(params.doubled_pawns.eg);
        }

        if self.meta[20].active {
            params.quadrupled_pawns.mg = params.quadrupled_pawns.mg.min(params.tripled_pawns.mg);
            params.quadrupled_pawns.eg = params.quadrupled_pawns.eg.min(params.tripled_pawns.eg);
        }

        // Material value
        if self.meta[8].active {
            params.bishop_value.mg = params.bishop_value.mg.max(params.knight_value.mg - 20);
            params.bishop_value.eg = params.bishop_value.eg.max(params.knight_value.eg - 20);
        }
        if self.meta[9].active {
            params.rook_value.mg = params.rook_value.mg.max(params.bishop_value.mg + 100);
            params.rook_value.eg = params.rook_value.eg.max(params.bishop_value.eg + 100);
        }

        if self.meta[7].active && self.meta[36].active {
            normalise_mean_zero(&mut params.knight_value, &mut params.knight_mobility);
        }
        if self.meta[8].active && self.meta[37].active {
            normalise_mean_zero(&mut params.bishop_value, &mut params.bishop_mobility);
        }
        if self.meta[9].active && self.meta[38].active {
            normalise_mean_zero(&mut params.rook_value, &mut params.rook_mobility);
        }
        if self.meta[10].active && self.meta[39].active {
            normalise_mean_zero(&mut params.queen_value, &mut params.queen_mobility);
        }

        if self.meta[8].active && self.meta[14].active {
            normalise_mean_zero(
                &mut params.bishop_value,
                &mut params.bishop_same_colour_pawns,
            );
        }

        if self.meta[33].active {
            normalise_king_ring(&mut params.king_ring_attacks);
        }

        self.clamp(params);
    }

    fn clamp(&self, params: &mut Self::ParamType) {
        let meta = self.flat_param_meta();
        let mut theta = self.pack(params);

        for i in 0..theta.len() {
            if meta[i].active {
                theta[i] = theta[i].clamp(meta[i].bounds.min, meta[i].bounds.max);
            }
        }

        *params = Self::unpack(&theta);
    }

    fn default_params(&self) -> Self::ParamType {
        self.params
    }
}

impl TuningConfig for LazyTuningConfig {
    type ParamType = LazyParams;

    fn pack(&self, params: &Self::ParamType) -> Vec<i32> {
        let mut out = Vec::new();

        push_pawn_pst(&mut out, &params.pawn_pst);
        push_score_array(&mut out, &params.knight_pst);
        push_score_array(&mut out, &params.bishop_pst);
        push_score_array(&mut out, &params.rook_pst);
        push_score_array(&mut out, &params.queen_pst);
        push_score_array(&mut out, &params.king_pst);

        push_score(&mut out, params.pawn_value);
        push_score(&mut out, params.knight_value);
        push_score(&mut out, params.bishop_value);
        push_score(&mut out, params.rook_value);
        push_score(&mut out, params.queen_value);

        out
    }

    fn unpack(values: &[i32]) -> Self::ParamType {
        let mut it = values.iter().copied();

        let params = Self::ParamType {
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

    fn flat_param_meta(&self) -> Vec<ParamMeta> {
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
        ] = self.meta;

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

    fn project(&self, params: &mut Self::ParamType) {
        // Material value
        if self.meta[8].active {
            params.bishop_value.mg = params.bishop_value.mg.max(params.knight_value.mg - 20);
            params.bishop_value.eg = params.bishop_value.eg.max(params.knight_value.eg - 20);
        }
        if self.meta[9].active {
            params.rook_value.mg = params.rook_value.mg.max(params.bishop_value.mg + 100);
            params.rook_value.eg = params.rook_value.eg.max(params.bishop_value.eg + 100);
        }

        self.clamp(params);
    }

    fn clamp(&self, params: &mut Self::ParamType) {
        let meta = self.flat_param_meta();
        let mut theta = self.pack(params);

        for i in 0..theta.len() {
            if meta[i].active {
                theta[i] = theta[i].clamp(meta[i].bounds.min, meta[i].bounds.max);
            }
        }

        *params = Self::unpack(&theta);
    }

    fn default_params(&self) -> Self::ParamType {
        self.params
    }
}

#[rustfmt::skip]
pub const DEFAULT_PARAM_META: [ParamMeta; PARAM_COUNT] = [
    m!(b!(-50, 200), true),  // 0 - pawn pst
    m!(b!(-200, 200), true), // 1 - knight pst
    m!(b!(-100, 100), true), // 2 - bishop pst
    m!(b!(-100, 100), true), // 3 - rook pst
    m!(b!(-200, 200), true), // 4 - queen pst
    m!(b!(-200, 200), true), // 5 - king pst

    m!(b!(70, 100), true),   // 6 - pawn value
    m!(b!(240, 360), true),  // 7 - knight value
    m!(b!(250, 370), true),  // 8 - bishop value
    m!(b!(400, 560), true),  // 9 - rook value
    m!(b!(850, 1100), true), // 10 - queen value

    m!(b!(-5, 30), true),    // 11 - knight outpost
    m!(b!(-5, 60), true),    // 12 - defended knight outpost

    m!(b!(-5, 100), true),   // 13 - bishop pair
    m!(b!(-50, 30), true),   // 14 - bishop same colour pawns
    m!(b!(-20, 40), true),   // 15 - fianchettoed bishop

    m!(b!(-5, 100), true),   // 16 - rook open file
    m!(b!(-5, 100), true),   // 17 - rook semi-open file

    m!(b!(-100, -1), true),  // 18 - doubled pawns
    m!(b!(-200, -1), true),  // 19 - tripled pawns
    m!(b!(-400, -50), true), // 20 - quadrupled pawns
    m!(b!(-50, 10), true),   // 21 - isolated pawn
    m!(b!(-5, 200), true),   // 22 - passed pawn

    m!(b!(-75, 5), true),    // 23 - king on open file
    m!(b!(-40, 40), true),   // 24 - king on semi open file
    m!(b!(-75, 0), true),    // 25 - king shield missing pawn
    m!(b!(-40, 40), true),   // 26 - king pawn shield distance
    m!(b!(-50, 0), true),    // 27 - enemy pawn distance from backrank

    m!(b!(1, 2), true),      // 28 - king ring pawn weights
    m!(b!(2, 3), true),      // 29 - king ring knight weights
    m!(b!(2, 3), true),      // 30 - king ring bishop weights
    m!(b!(3, 4), true),      // 31 - king ring rook weights
    m!(b!(4, 5), true),      // 32 - king ring queen weights
    m!(b!(-200, 20), true),  // 33 - king ring attacks

    m!(b!(-60, 60), true),   // 34 - knight adj
    m!(b!(-60, 60), true),   // 35 - rook adj

    m!(b!(-75, 75), true),   // 36 - knight mobility
    m!(b!(-75, 75), true),   // 37 - bishop mobility
    m!(b!(-50, 50), true),   // 38 - rook mobility
    m!(b!(-50, 50), true),   // 39 - queen mobility
];

#[rustfmt::skip]
pub const DEFAULT_LAZY_PARAM_META: [ParamMeta; LAZY_PARAM_COUNT] = [
    m!(b!(-50, 200), true),  // 0 - pawn pst
    m!(b!(-200, 200), true), // 1 - knight pst
    m!(b!(-100, 100), true), // 2 - bishop pst
    m!(b!(-100, 100), true), // 3 - rook pst
    m!(b!(-200, 200), true), // 4 - queen pst
    m!(b!(-200, 200), true), // 5 - king pst

    m!(b!(70, 100), true),   // 6 - pawn value
    m!(b!(240, 360), true),  // 7 - knight value
    m!(b!(250, 370), true),  // 8 - bishop value
    m!(b!(400, 560), true),  // 9 - rook value
    m!(b!(850, 1100), true), // 10 - queen value
];
