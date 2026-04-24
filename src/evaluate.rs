use std::ops::{Add, Div, Mul};

use crate::{
    bitboard::{Bitboard, Bitboards, bitboards},
    params::{DEFAULT_LAZY_PARAMS, DEFAULT_PARAMS, LazyParams, PST, Params},
    position::Position,
    types::{Black, Colour, Direction, Piece, Side, Square, White},
};

pub type Eval = i32;
pub const INFINITY: Eval = 32001;
pub const NO_EVAL: i16 = i16::MIN;

const PHASE_WEIGHTS: [u32; 6] = [0, 1, 1, 2, 4, 0];

#[derive(Copy, Clone, Debug, Default)]
pub struct Score {
    pub mg: Eval,
    pub eg: Eval,
}

impl Score {
    fn add<S: Side>(&mut self, score: Score) {
        let sign = if S::IS_WHITE { 1 } else { -1 };
        self.mg += sign * score.mg;
        self.eg += sign * score.eg;
    }
}

impl Add for Score {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            mg: self.mg + rhs.mg,
            eg: self.eg + rhs.eg,
        }
    }
}

impl Mul<i32> for Score {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self {
        Self {
            mg: self.mg * rhs,
            eg: self.eg * rhs,
        }
    }
}

impl Div<i32> for Score {
    type Output = Self;
    fn div(self, rhs: i32) -> Self {
        Self {
            mg: self.mg / rhs,
            eg: self.eg / rhs,
        }
    }
}

#[derive(Default, Copy, Clone)]
struct EvalInfo {
    king_ring: [Bitboard; 2],
    pawn_attacks: [Bitboard; 2],
    knight_attacks: [Bitboard; 2],
    bishop_attacks: [Bitboard; 2],
    rook_attacks: [Bitboard; 2],
    queen_attacks: [Bitboard; 2],
    all_attacks: [Bitboard; 2],
    king_ring_knight_hits: [u32; 2],
    king_ring_bishop_hits: [u32; 2],
    king_ring_rook_hits: [u32; 2],
    king_ring_queen_hits: [u32; 2],
}

impl EvalInfo {
    fn init(pos: &Position, bbs: &Bitboards) -> Self {
        let white_king_ring = bbs.king_attacks(pos.white_king_square);
        let black_king_ring = bbs.king_attacks(pos.black_king_square);

        Self {
            king_ring: [white_king_ring, black_king_ring],
            pawn_attacks: [Bitboard::default(), Bitboard::default()],
            knight_attacks: [Bitboard::default(), Bitboard::default()],
            bishop_attacks: [Bitboard::default(), Bitboard::default()],
            rook_attacks: [Bitboard::default(), Bitboard::default()],
            queen_attacks: [Bitboard::default(), Bitboard::default()],
            all_attacks: [white_king_ring, black_king_ring],
            king_ring_knight_hits: [0, 0],
            king_ring_bishop_hits: [0, 0],
            king_ring_rook_hits: [0, 0],
            king_ring_queen_hits: [0, 0],
        }
    }
}

#[inline(always)]
const fn phase_weight(piece: Piece) -> u32 {
    PHASE_WEIGHTS[piece.idx()]
}

#[inline(always)]
const fn relative_square<S: Side>(sq: Square) -> usize {
    match S::COLOUR {
        Colour::Black => sq.idx(),
        Colour::White => sq.idx() ^ 56,
    }
}

#[inline(always)]
const fn relative_rank<S: Side>(sq: Square) -> u8 {
    match S::COLOUR {
        Colour::White => sq.rank(),
        Colour::Black => 7 - sq.rank(),
    }
}

fn game_phase(pos: &Position) -> i32 {
    let mut phase = 0;
    let white = pos.pieces[Colour::White.idx()];
    let black = pos.pieces[Colour::Black.idx()];

    phase += (white[Piece::Pawn.idx()] | black[Piece::Pawn.idx()]).bit_count()
        * phase_weight(Piece::Pawn);
    phase += (white[Piece::Knight.idx()] | black[Piece::Knight.idx()]).bit_count()
        * phase_weight(Piece::Knight);
    phase += (white[Piece::Bishop.idx()] | black[Piece::Bishop.idx()]).bit_count()
        * phase_weight(Piece::Bishop);
    phase += (white[Piece::Rook.idx()] | black[Piece::Rook.idx()]).bit_count()
        * phase_weight(Piece::Rook);
    phase += (white[Piece::Queen.idx()] | black[Piece::Queen.idx()]).bit_count()
        * phase_weight(Piece::Queen);

    phase as i32
}

fn evaluate_pawns<S: Side>(
    pos: &Position,
    score: &mut Score,
    info: &mut EvalInfo,
    params: &Params,
) {
    let bbs = bitboards();

    let pawns = pos.pieces[S::IDX][Piece::Pawn.idx()];
    let opp_pawns = pos.pieces[S::THEM][Piece::Pawn.idx()];

    let (up, down) = if S::IS_WHITE {
        (Direction::North, Direction::South)
    } else {
        (Direction::South, Direction::North)
    };

    let opp_double_attacks = opp_pawns.pawn_double_attacks_bb::<S::Opp>();

    // Material eval
    score.add::<S>(params.pawn_value * pawns.bit_count() as i32);

    // Store pawn attacks
    let pawn_attacks = if S::IS_WHITE {
        pawns.shift(Direction::NorthEast) | pawns.shift(Direction::NorthWest)
    } else {
        pawns.shift(Direction::SouthEast) | pawns.shift(Direction::SouthWest)
    };
    info.pawn_attacks[S::IDX] = pawn_attacks;
    info.all_attacks[S::IDX] |= pawn_attacks;

    // Iterate over every pawn
    let mut pawn_bb = pawns;
    while !pawn_bb.is_empty() {
        let sq = pawn_bb.pop_lsb();
        let rel_sq = relative_square::<S>(sq);

        // PST eval
        score.add::<S>(params.pawn_pst[rel_sq]);

        let sq_bb = sq.bitboard();
        let rel_rank = relative_rank::<S>(sq);
        let rank_bb = sq.rank_bb();
        let file_bb = sq.file_bb();

        let front = sq_bb.frontfill(S::COLOUR) ^ sq_bb;
        let passed_pawn_span = front | front.shift(Direction::East) | front.shift(Direction::West);
        let adj_files = file_bb.shift(Direction::East) | file_bb.shift(Direction::West);

        // Flags for the pawn
        let frontmost = (pawns & front).is_empty();
        let opposed = !(opp_pawns & front).is_empty();
        let blocked = opp_pawns & sq_bb.shift(up);
        let stoppers = opp_pawns & passed_pawn_span;
        let lever = opp_pawns & bbs.pawn_attacks(sq, S::COLOUR);
        let lever_push = opp_pawns & bbs.pawn_attacks(sq_bb.shift(up).lsb(), S::COLOUR);
        let neighbours = pawns & adj_files;
        let phalanx = neighbours & rank_bb;
        let support = neighbours & rank_bb.shift(down);

        // A pawn is backward when it is behind all pawns of the same colour
        // on the adjacent files and cannot safely advance
        let rear = sq_bb.backfill(S::COLOUR);
        let backward_span = rear | rear.shift(Direction::East) | rear.shift(Direction::West);
        let backward = !neighbours.is_empty()
            && (neighbours & backward_span).is_empty()
            && !(lever_push | blocked).is_empty();

        let passed = stoppers.is_empty();

        // A pawn is a candidated passer if it is not a passer and if one of the three following conditions is true:
        // 1. There are no stoppers except some levers
        // 2. The only stoppers are the lever_push, but we outnumber them
        // 3. There is only one front stopper which can be levered
        let candidate = (stoppers ^ lever).is_empty()
            || ((stoppers ^ lever_push).is_empty()
                && phalanx.bit_count() >= lever_push.bit_count())
            || (stoppers == blocked
                && rel_rank >= 4
                && !(support.shift(up) & !(opp_pawns | opp_double_attacks)).is_empty());

        if passed {
            score.add::<S>(params.passed_pawn[rel_rank as usize - 1]);
        } else if candidate {
            score.add::<S>(params.candidate_passer[rel_rank as usize - 1]);
        }

        // Score support/phalanx pawns
        if !support.is_empty() || !phalanx.is_empty() {
            let bonus = params.connected_bonus[rel_rank as usize - 1]
                * (2 + !phalanx.is_empty() as i32 - opposed as i32)
                + params.supported_bonus[support.bit_count() as usize];
            score.add::<S>(bonus);
        }
        // Isolated pawn
        else if neighbours.is_empty() && frontmost {
            let bucket = sq.file().min(7 - sq.file()) as usize;
            let penalty = params.isolated_pawn[bucket] + params.weak_unopposed * !opposed as i32;
            score.add::<S>(penalty);
        }
        // Backward pawn
        else if backward {
            let bucket = sq.file().min(7 - sq.file()) as usize;
            let penalty = params.backward_pawn[bucket] + params.weak_unopposed * !opposed as i32;
            score.add::<S>(penalty);
        }

        // Doubled/tripled/etc pawns
        if frontmost {
            match (pawns & file_bb).bit_count().min(4) {
                1 => {}
                2 => score.add::<S>(params.doubled_pawns),
                3 => score.add::<S>(params.tripled_pawns),
                4 => score.add::<S>(params.quadrupled_pawns),
                _ => unreachable!(),
            }
        }
    }
}

fn evaluate_knights<S: Side>(
    pos: &Position,
    score: &mut Score,
    info: &mut EvalInfo,
    params: &Params,
) {
    let bbs = bitboards();

    let own_occ = pos.occupancy[S::IDX];
    let pawns = pos.pieces[S::IDX][Piece::Pawn.idx()];
    let pawn_count = pawns.bit_count() as usize;

    let mut knights = pos.pieces[S::IDX][Piece::Knight.idx()];

    // Material eval
    score.add::<S>(params.knight_value * knights.bit_count() as i32);

    while !knights.is_empty() {
        let knight = knights.pop_lsb();

        // PST eval
        let sq = relative_square::<S>(knight);
        score.add::<S>(params.knight_pst[sq]);

        // Mobility/attacks
        let control = bbs.knight_attacks(knight);
        let attacks = control & !own_occ;
        info.king_ring_knight_hits[S::THEM] += (control & info.king_ring[S::THEM]).bit_count();
        info.knight_attacks[S::IDX] |= control;
        info.all_attacks[S::IDX] |= control;
        let mobility = attacks.bit_count() as usize;

        score.add::<S>(params.knight_mobility[mobility]);

        // Adjust knight value based on number of pawns remaining
        score.add::<S>(params.knight_adj[pawn_count]);

        // Knight outposts
        // Only care about knights in opponent half of the board
        let rank = relative_rank::<S>(knight);
        if rank < 4 {
            continue;
        }

        // Opponent pawns on adjacent files in front of the knight can challenge it
        let front = knight.bitboard().frontfill(S::COLOUR) ^ knight.bitboard();
        let adj_files = front.shift(Direction::East) | front.shift(Direction::West);
        let challengers = pos.pieces[S::THEM][Piece::Pawn.idx()] & adj_files;

        if challengers.is_empty() {
            // If own pawn supports the outpost
            if !(bbs.pawn_attacks(knight, S::COLOUR.opposite()) & pawns).is_empty() {
                score.add::<S>(params.defended_knight_outpost);
            } else {
                score.add::<S>(params.knight_outpost);
            }
        }
    }
}

fn evaluate_bishops<S: Side>(
    pos: &Position,
    score: &mut Score,
    info: &mut EvalInfo,
    params: &Params,
) {
    let bbs = bitboards();

    let own_occ = pos.occupancy[S::IDX];
    let occ = pos.occupancy[2];

    let mut bishops = pos.pieces[S::IDX][Piece::Bishop.idx()];

    // Material eval
    let bishop_count = bishops.bit_count();
    score.add::<S>(params.bishop_value * bishop_count as i32);

    let pawns = pos.pieces[S::IDX][Piece::Pawn.idx()];
    let light_pawns = (pawns & Bitboard::LIGHT_SQUARES).bit_count() as usize;
    let dark_pawns = (pawns & Bitboard::DARK_SQUARES).bit_count() as usize;

    // Bishop pair
    if bishop_count > 1 {
        score.add::<S>(params.bishop_pair);
    }

    while !bishops.is_empty() {
        let bishop = bishops.pop_lsb();

        // PST eval
        let sq = relative_square::<S>(bishop);
        score.add::<S>(params.bishop_pst[sq]);

        // Mobility/attacks
        let control = bbs.bishop_attacks(bishop, occ);
        let attacks = control & !own_occ;
        info.king_ring_bishop_hits[S::THEM] += (control & info.king_ring[S::THEM]).bit_count();
        info.bishop_attacks[S::IDX] |= control;
        info.all_attacks[S::IDX] |= control;
        let mobility = attacks.bit_count() as usize;

        score.add::<S>(params.bishop_mobility[mobility]);

        // Fianchettoed bishop
        if S::IS_WHITE {
            if bishop == Square::B2 && !(pawns & Square::B3.bitboard()).is_empty() {
                score.add::<S>(params.fianchetto);
            }
            if bishop == Square::G2 && !(pawns & Square::G3.bitboard()).is_empty() {
                score.add::<S>(params.fianchetto);
            }
        } else {
            if bishop == Square::B7 && !(pawns & Square::B6.bitboard()).is_empty() {
                score.add::<S>(params.fianchetto);
            }
            if bishop == Square::G7 && !(pawns & Square::G6.bitboard()).is_empty() {
                score.add::<S>(params.fianchetto);
            }
        }

        // Good/bad bishop eval
        if bishop.colour() == Colour::White {
            score.add::<S>(params.bishop_same_colour_pawns[light_pawns]);
        } else {
            score.add::<S>(params.bishop_same_colour_pawns[dark_pawns]);
        }
    }
}

fn evaluate_rooks<S: Side>(
    pos: &Position,
    score: &mut Score,
    info: &mut EvalInfo,
    params: &Params,
) {
    let bbs = bitboards();

    let own_occ = pos.occupancy[S::IDX];
    let occ = pos.occupancy[2];

    let own_pawns = pos.pieces[S::IDX][Piece::Pawn.idx()];
    let opp_pawns = pos.pieces[S::THEM][Piece::Pawn.idx()];

    let pawn_count = own_pawns.bit_count() as usize;

    let mut rooks = pos.pieces[S::IDX][Piece::Rook.idx()];

    // Material eval
    score.add::<S>(params.rook_value * rooks.bit_count() as i32);

    while !rooks.is_empty() {
        let rook = rooks.pop_lsb();
        let file = rook.file();

        // PST eval
        let sq = relative_square::<S>(rook);
        score.add::<S>(params.rook_pst[sq]);

        // Mobility/attacks
        let control = bbs.rook_attacks(rook, occ);
        let attacks = control & !own_occ;
        info.king_ring_rook_hits[S::THEM] += (control & info.king_ring[S::THEM]).bit_count();
        info.rook_attacks[S::IDX] |= control;
        info.all_attacks[S::IDX] |= control;
        let mobility = attacks.bit_count() as usize;

        score.add::<S>(params.rook_mobility[mobility]);

        // Open/semi open files
        let own_on_file = own_pawns.file_occupied(file);
        let opp_on_file = opp_pawns.file_occupied(file);

        if !own_on_file {
            if !opp_on_file {
                score.add::<S>(params.rook_open_file);
            } else {
                score.add::<S>(params.rook_semi_open_file)
            }
        }

        // Adjust rook value based on number of pawns remaining
        score.add::<S>(params.rook_adj[pawn_count]);
    }
}

fn evaluate_queens<S: Side>(
    pos: &Position,
    score: &mut Score,
    info: &mut EvalInfo,
    params: &Params,
) {
    let bbs = bitboards();

    let home = if S::IS_WHITE { Square::D1 } else { Square::D8 };

    let own_occ = pos.occupancy[S::IDX];
    let occ = pos.occupancy[2];

    let mut queens = pos.pieces[S::IDX][Piece::Queen.idx()];

    // Material eval
    score.add::<S>(params.queen_value * queens.bit_count() as i32);

    while !queens.is_empty() {
        let queen = queens.pop_lsb();

        // PST eval
        let sq = relative_square::<S>(queen);
        score.add::<S>(params.queen_pst[sq]);

        // Punish early development
        if queen != home {
            let backrank = if S::IS_WHITE {
                Bitboard::new(0x66)
            } else {
                Bitboard::new(0x66 << 56)
            };
            let pieces =
                pos.pieces[S::IDX][Piece::Knight.idx()] | pos.pieces[S::IDX][Piece::Bishop.idx()];
            let undeveloped = (backrank & pieces).bit_count() as i32;

            if undeveloped > 0 {
                let king_on_home_sq = if S::IS_WHITE {
                    (pos.white_king_square == Square::E1) as i32
                } else {
                    (pos.black_king_square == Square::E8) as i32
                };

                score.add::<S>(Score {
                    mg: params.queen_undeveloped_piece_punishment.mg * undeveloped
                        + params.queen_unmoved_king_punishment.mg * king_on_home_sq,
                    eg: 0,
                });
            }
        }

        // Mobility/attacks
        let control = bbs.bishop_attacks(queen, occ) | bbs.rook_attacks(queen, occ);
        let attacks = control & !own_occ;
        info.king_ring_queen_hits[S::THEM] += (control & info.king_ring[S::THEM]).bit_count();
        info.queen_attacks[S::IDX] |= control;
        info.all_attacks[S::IDX] |= control;
        let mobility = attacks.bit_count() as usize;

        score.add::<S>(params.queen_mobility[mobility]);
    }
}

fn evaluate_king_safety<S: Side>(
    pos: &Position,
    score: &mut Score,
    info: &mut EvalInfo,
    params: &Params,
) {
    let king = if S::IS_WHITE {
        pos.white_king_square
    } else {
        pos.black_king_square
    };

    // PST evaluation
    let sq = relative_square::<S>(king);
    score.add::<S>(params.king_pst[sq]);

    let own_pawns = pos.pieces[S::IDX][Piece::Pawn.idx()];
    let opp_pawns = pos.pieces[S::THEM][Piece::Pawn.idx()];

    let file_bb = king.file_bb();
    let file = king.file();

    // King on open/semi open files
    let own_on_file = own_pawns.file_occupied(file);
    let opp_on_file = opp_pawns.file_occupied(file);

    if !own_on_file {
        if !opp_on_file {
            score.add::<S>(params.king_on_open_file);
        } else {
            score.add::<S>(params.king_on_semi_open_file);
        }
    }

    // King ring attacks
    let mut mg_units = 0;
    let mut eg_units = 0;

    let w = params.king_ring_pawn_weight;
    let hits = (info.pawn_attacks[S::THEM] & info.king_ring[S::IDX]).bit_count() as i32;
    mg_units += w.mg * hits;
    eg_units += w.eg * hits;

    let w = params.king_ring_knight_weight;
    let hits = info.king_ring_knight_hits[S::IDX] as i32;
    mg_units += w.mg * hits;
    eg_units += w.eg * hits;

    let w = params.king_ring_bishop_weight;
    let hits = info.king_ring_bishop_hits[S::IDX] as i32;
    mg_units += w.mg * hits;
    eg_units += w.eg * hits;

    let w = params.king_ring_rook_weight;
    let hits = info.king_ring_rook_hits[S::IDX] as i32;
    mg_units += w.mg * hits;
    eg_units += w.eg * hits;

    let w = params.king_ring_queen_weight;
    let hits = info.king_ring_queen_hits[S::IDX] as i32;
    mg_units += w.mg * hits;
    eg_units += w.eg * hits;

    let attacks_value = Score {
        mg: params.king_ring_attacks[mg_units.min(23) as usize].mg,
        eg: params.king_ring_attacks[eg_units.min(23) as usize].eg,
    };
    score.add::<S>(attacks_value);

    // King pawn shield
    let backrank: u8 = if S::IS_WHITE { 0 } else { 7 };

    let mut shield_count = 0;
    let mut pawn_shield =
        own_pawns & (file_bb | file_bb.shift(Direction::East) | file_bb.shift(Direction::West));
    while !pawn_shield.is_empty() {
        let pawn = pawn_shield.pop_lsb();
        let d = backrank.abs_diff(pawn.rank()).min(4) as usize - 1;

        score.add::<S>(params.king_pawn_shield_distance[d]);
        shield_count += 1;
    }
    if shield_count < 3 {
        score.add::<S>(params.king_shield_missing_pawn * (3 - shield_count));
    }

    // Enemy pawn storm
    let mut pawn_storm =
        opp_pawns & (file_bb | file_bb.shift(Direction::East) | file_bb.shift(Direction::West));
    while !pawn_storm.is_empty() {
        let pawn = pawn_storm.pop_lsb();
        let d = backrank.abs_diff(pawn.rank()).min(4) as usize - 1;

        score.add::<S>(params.enemy_pawn_distance_from_backrank[d]);
    }
}

fn evaluate_threats<S: Side>(pos: &Position, score: &mut Score, info: &EvalInfo, params: &Params) {
    let their_minors =
        pos.pieces[S::THEM][Piece::Knight.idx()] | pos.pieces[S::THEM][Piece::Bishop.idx()];
    let their_majors =
        pos.pieces[S::THEM][Piece::Rook.idx()] | pos.pieces[S::THEM][Piece::Queen.idx()];
    let their_rooks = pos.pieces[S::THEM][Piece::Rook.idx()];
    let their_queen = pos.pieces[S::THEM][Piece::Queen.idx()];

    let pawn_threat_minors = their_minors & info.pawn_attacks[S::IDX];
    let pawn_threat_majors = their_majors & info.pawn_attacks[S::IDX];

    let hanging_minors = their_minors & info.all_attacks[S::IDX] & !info.all_attacks[S::THEM];
    let hanging_rooks = their_rooks & info.all_attacks[S::IDX] & !info.all_attacks[S::THEM];
    let hanging_queen = their_queen & info.all_attacks[S::IDX] & !info.all_attacks[S::THEM];

    let queen_hit_by_minor =
        their_queen & (info.knight_attacks[S::IDX] | info.bishop_attacks[S::IDX]);
    let queen_hit_by_rook = their_queen & info.rook_attacks[S::IDX];

    score.add::<S>(params.pawn_threat_minor * pawn_threat_minors.bit_count().min(2) as i32);
    score.add::<S>(params.pawn_threat_major * pawn_threat_majors.bit_count().min(2) as i32);
    score.add::<S>(params.hanging_minor * hanging_minors.bit_count().min(2) as i32);
    score.add::<S>(params.hanging_rook * hanging_rooks.bit_count().min(2) as i32);

    if !hanging_queen.is_empty() {
        score.add::<S>(params.hanging_queen);
    } else if !queen_hit_by_rook.is_empty() {
        score.add::<S>(params.rook_threat_queen);
    } else if !queen_hit_by_minor.is_empty() {
        score.add::<S>(params.minor_threat_queen);
    }
}

fn taper(score: Score, phase: i32, us: Colour) -> Eval {
    let mg_phase = phase.min(24);
    let eg_phase = 24 - mg_phase;

    let score = (mg_phase * score.mg + eg_phase * score.eg) / 24;

    if us == Colour::White { score } else { -score }
}

pub fn evaluate(pos: &Position) -> Eval {
    evaluate_with(pos, &DEFAULT_PARAMS)
}

pub fn evaluate_with(pos: &Position, params: &Params) -> Eval {
    let bbs = bitboards();

    let mut score = Score::default();
    let mut info = EvalInfo::init(pos, bbs);
    let phase = game_phase(pos);

    evaluate_pawns::<White>(pos, &mut score, &mut info, params);
    evaluate_pawns::<Black>(pos, &mut score, &mut info, params);

    evaluate_knights::<White>(pos, &mut score, &mut info, params);
    evaluate_knights::<Black>(pos, &mut score, &mut info, params);

    evaluate_bishops::<White>(pos, &mut score, &mut info, params);
    evaluate_bishops::<Black>(pos, &mut score, &mut info, params);

    evaluate_rooks::<White>(pos, &mut score, &mut info, params);
    evaluate_rooks::<Black>(pos, &mut score, &mut info, params);

    evaluate_queens::<White>(pos, &mut score, &mut info, params);
    evaluate_queens::<Black>(pos, &mut score, &mut info, params);

    evaluate_threats::<White>(pos, &mut score, &mut info, params);
    evaluate_threats::<Black>(pos, &mut score, &mut info, params);

    evaluate_king_safety::<White>(pos, &mut score, &mut info, params);
    evaluate_king_safety::<Black>(pos, &mut score, &mut info, params);

    taper(score, phase, pos.side_to_move)
}

fn lazy_piece_terms<S: Side>(mut bb: Bitboard, value: Option<Score>, pst: &PST, score: &mut Score) {
    // Add (non-king) piece values to score
    if let Some(value) = value {
        score.add::<S>(value * bb.bit_count() as i32);
    }

    // Add PST weights
    while !bb.is_empty() {
        let sq = relative_square::<S>(bb.pop_lsb());
        score.add::<S>(pst[sq]);
    }
}

pub fn lazy_evaluate(pos: &Position) -> Eval {
    lazy_evaluate_with(pos, &DEFAULT_LAZY_PARAMS)
}

pub fn lazy_evaluate_with(pos: &Position, params: &LazyParams) -> Eval {
    let mut score = Score::default();
    let phase = game_phase(pos);

    lazy_piece_terms::<White>(
        pos.pieces[White::IDX][Piece::Pawn.idx()],
        Some(params.pawn_value),
        &params.pawn_pst,
        &mut score,
    );
    lazy_piece_terms::<Black>(
        pos.pieces[Black::IDX][Piece::Pawn.idx()],
        Some(params.pawn_value),
        &params.pawn_pst,
        &mut score,
    );

    lazy_piece_terms::<White>(
        pos.pieces[White::IDX][Piece::Knight.idx()],
        Some(params.knight_value),
        &params.knight_pst,
        &mut score,
    );
    lazy_piece_terms::<Black>(
        pos.pieces[Black::IDX][Piece::Knight.idx()],
        Some(params.knight_value),
        &params.knight_pst,
        &mut score,
    );

    lazy_piece_terms::<White>(
        pos.pieces[White::IDX][Piece::Bishop.idx()],
        Some(params.bishop_value),
        &params.bishop_pst,
        &mut score,
    );
    lazy_piece_terms::<Black>(
        pos.pieces[Black::IDX][Piece::Bishop.idx()],
        Some(params.bishop_value),
        &params.bishop_pst,
        &mut score,
    );

    lazy_piece_terms::<White>(
        pos.pieces[White::IDX][Piece::Rook.idx()],
        Some(params.rook_value),
        &params.rook_pst,
        &mut score,
    );
    lazy_piece_terms::<Black>(
        pos.pieces[Black::IDX][Piece::Rook.idx()],
        Some(params.rook_value),
        &params.rook_pst,
        &mut score,
    );

    lazy_piece_terms::<White>(
        pos.pieces[White::IDX][Piece::Queen.idx()],
        Some(params.queen_value),
        &params.queen_pst,
        &mut score,
    );
    lazy_piece_terms::<Black>(
        pos.pieces[Black::IDX][Piece::Queen.idx()],
        Some(params.queen_value),
        &params.queen_pst,
        &mut score,
    );

    lazy_piece_terms::<White>(
        pos.pieces[White::IDX][Piece::King.idx()],
        None,
        &params.king_pst,
        &mut score,
    );
    lazy_piece_terms::<Black>(
        pos.pieces[Black::IDX][Piece::King.idx()],
        None,
        &params.king_pst,
        &mut score,
    );

    taper(score, phase, pos.side_to_move)
}
