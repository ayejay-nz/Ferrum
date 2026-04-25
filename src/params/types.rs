use crate::evaluate::Score;
use crate::params::values::*;

pub type PST = [Score; 64];

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
    pub fianchetto: Score,
    pub bishop_outpost: Score,
    pub defended_bishop_outpost: Score,

    pub rook_open_file: Score,
    pub rook_semi_open_file: Score,
    pub rook_on_seventh: Score,
    pub rook_on_queen_file: Score,
    pub connected_doubled_rooks: Score,

    pub queen_undeveloped_piece_punishment: Score,
    pub queen_unmoved_king_punishment: Score,

    pub pawn_threat_minor: Score,
    pub pawn_threat_major: Score,
    pub hanging_minor: Score,
    pub hanging_rook: Score,
    pub hanging_queen: Score,
    pub minor_threat_queen: Score,
    pub rook_threat_queen: Score,

    pub doubled_pawns: Score,
    pub tripled_pawns: Score,
    pub quadrupled_pawns: Score,
    pub isolated_pawn: [Score; 4],
    pub backward_pawn: [Score; 4],
    pub weak_unopposed: Score,
    pub candidate_passer: [Score; 6],
    pub connected_bonus: [Score; 6],
    pub supported_bonus: [Score; 3],
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
    pub king_virtual_mobility: [Score; 28],

    pub knight_adj: [Score; 9],
    pub rook_adj: [Score; 9],

    pub knight_mobility: [Score; 9],
    pub bishop_mobility: [Score; 14],
    pub rook_mobility: [Score; 15],
    pub queen_mobility: [Score; 28],
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
    fianchetto: FIANCHETTO,
    bishop_outpost: BISHOP_OUTPOST,
    defended_bishop_outpost: DEFENDED_BISHOP_OUTPOST,

    rook_open_file: ROOK_OPEN_FILE,
    rook_semi_open_file: ROOK_SEMI_OPEN_FILE,
    rook_on_seventh: ROOK_ON_SEVENTH,
    rook_on_queen_file: ROOK_ON_QUEEN_FILE,
    connected_doubled_rooks: CONNECTED_DOUBLED_ROOKS,

    queen_undeveloped_piece_punishment: QUEEN_UNDEVELOPED_PIECE_PUNISHMENT,
    queen_unmoved_king_punishment: QUEEN_UNMOVED_KING_PUNISHMENT,

    pawn_threat_minor: PAWN_THREAT_MINOR,
    pawn_threat_major: PAWN_THREAT_MAJOR,
    hanging_minor: HANGING_MINOR,
    hanging_rook: HANGING_ROOK,
    hanging_queen: HANGING_QUEEN,
    minor_threat_queen: MINOR_THREAT_QUEEN,
    rook_threat_queen: ROOK_THREAT_QUEEN,

    doubled_pawns: DOUBLED_PAWNS,
    tripled_pawns: TRIPLED_PAWNS,
    quadrupled_pawns: QUADRUPLED_PAWNS,
    isolated_pawn: ISOLATED_PAWN,
    backward_pawn: BACKWARD_PAWN,
    weak_unopposed: WEAK_UNOPPOSED,
    candidate_passer: CANDIDATE_PASSER,
    connected_bonus: CONNECTED_BONUS,
    supported_bonus: SUPPORTED_BONUS,
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
    king_virtual_mobility: KING_VIRTUAL_MOBILITY,

    knight_adj: KNIGHT_ADJ,
    rook_adj: ROOK_ADJ,

    knight_mobility: KNIGHT_MOBILITY,
    bishop_mobility: BISHOP_MOBILITY,
    rook_mobility: ROOK_MOBILITY,
    queen_mobility: QUEEN_MOBILITY,
};

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
