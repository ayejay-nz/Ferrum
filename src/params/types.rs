use crate::evaluate::Score;

use crate::params::helpers::*;
use crate::params::values::*;

macro_rules! b {
    ($min:literal, $max:literal) => {
        ParamBounds {
            min: $min,
            max: $max,
        }
    };
}

pub type PST = [Score; 64];

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
