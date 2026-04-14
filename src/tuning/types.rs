use std::fmt;

use crate::{
    params::{DEFAULT_LAZY_PARAMS, DEFAULT_PARAMS, LAZY_PAWN_PST, LazyParams, PAWN_PST, Params},
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
