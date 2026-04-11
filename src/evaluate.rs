use std::ops::{Div, Mul};

use crate::{
    bitboard::{Bitboard, bitboards},
    position::Position,
    tune::{DEFAULT_LAZY_PARAMS, DEFAULT_PARAMS, LazyParams, Params},
    types::{Black, Colour, Direction, Piece, Side, Square, White},
};

pub type Eval = i32;
pub const INFINITY: Eval = 32001;

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

fn evaluate_pawns<S: Side>(pos: &Position, score: &mut Score, params: &Params) {
    let pawns = pos.pieces[S::IDX][Piece::Pawn.idx()];
    let opp_pawns = pos.pieces[S::THEM][Piece::Pawn.idx()];

    // Material eval
    score.add::<S>(params.pawn_value * pawns.bit_count() as i32);

    let mut file_counts = [0u8; 8];

    // PST eval
    let mut pawn_bb = pawns;
    while !pawn_bb.is_empty() {
        let pawn = pawn_bb.pop_lsb();
        let sq = relative_square::<S>(pawn);
        score.add::<S>(params.pawn_pst[sq]);

        file_counts[pawn.file() as usize] += 1;
    }

    // Find passed pawns
    let stoppers =
        (opp_pawns | opp_pawns.shift(Direction::East) | opp_pawns.shift(Direction::West))
            .backfill(S::COLOUR);

    let mut passers = pawns & !stoppers;

    while !passers.is_empty() {
        let sq = passers.pop_lsb();
        let rank = if S::IS_WHITE {
            sq.rank() as usize - 1
        } else {
            7 - sq.rank() as usize - 1
        };

        score.add::<S>(params.passed_pawn[rank]);
    }

    // Scan each file for isolated/multiple pawns
    for file in 0..8 {
        let pawn_count = file_counts[file].min(4);

        // Check for double/tripled/etc
        if pawn_count > 1 {
            match pawn_count {
                2 => score.add::<S>(params.doubled_pawns),
                3 => score.add::<S>(params.tripled_pawns),
                4 => score.add::<S>(params.quadrupled_pawns),
                _ => unreachable!(),
            }
        }

        // Check if a pawn is isolated
        let left = file.checked_sub(1).map(|f| file_counts[f]).unwrap_or(0);
        let right = file_counts.get(file + 1).copied().unwrap_or(0);

        if pawn_count > 0 && left == 0 && right == 0 {
            let bucket = file.min(7 - file);
            score.add::<S>(params.isolated_pawn[bucket]);
        }
    }
}

fn evaluate_knights<S: Side>(pos: &Position, score: &mut Score, params: &Params) {
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

        // Mobility
        let attacks = bbs.knight_attacks(knight) & !own_occ;
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

fn evaluate_bishops<S: Side>(pos: &Position, score: &mut Score, params: &Params) {
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

        // Mobility
        let attacks = bbs.bishop_attacks(bishop, occ) & !own_occ;
        let mobility = attacks.bit_count() as usize;

        score.add::<S>(params.bishop_mobility[mobility]);

        // Good/bad bishop eval
        if bishop.colour() == Colour::White {
            score.add::<S>(params.bishop_same_colour_pawns[light_pawns]);
        } else {
            score.add::<S>(params.bishop_same_colour_pawns[dark_pawns]);
        }
    }
}

fn evaluate_rooks<S: Side>(pos: &Position, score: &mut Score, params: &Params) {
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

        // Mobility
        let attacks = bbs.rook_attacks(rook, occ) & !own_occ;
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

fn evaluate_queens<S: Side>(pos: &Position, score: &mut Score, params: &Params) {
    let bbs = bitboards();

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

        // Mobility
        let attacks = (bbs.bishop_attacks(queen, occ) | bbs.rook_attacks(queen, occ)) & !own_occ;
        let mobility = attacks.bit_count() as usize;

        score.add::<S>(params.queen_mobility[mobility]);
    }
}

fn evaluate_king_safety<S: Side>(pos: &Position, score: &mut Score, params: &Params) {
    let bbs = bitboards();

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
    let occ = pos.occupancy[2];
    let enemy = S::COLOUR.opposite();

    let mut attacked = 0;
    let mut king_ring = bbs.king_attacks(king);
    while !king_ring.is_empty() {
        let sq = king_ring.pop_lsb();
        if !pos.attackers_to_by(sq, enemy, occ).is_empty() {
            attacked += 1;
        }
    }

    score.add::<S>(params.king_ring_attacks[attacked.min(4)]);

    let backrank: u8 = if S::IS_WHITE { 0 } else { 7 };

    // King pawn shield
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
    let mut score = Score::default();
    let phase = game_phase(pos);

    evaluate_pawns::<White>(pos, &mut score, params);
    evaluate_pawns::<Black>(pos, &mut score, params);

    evaluate_knights::<White>(pos, &mut score, params);
    evaluate_knights::<Black>(pos, &mut score, params);

    evaluate_bishops::<White>(pos, &mut score, params);
    evaluate_bishops::<Black>(pos, &mut score, params);

    evaluate_rooks::<White>(pos, &mut score, params);
    evaluate_rooks::<Black>(pos, &mut score, params);

    evaluate_queens::<White>(pos, &mut score, params);
    evaluate_queens::<Black>(pos, &mut score, params);

    evaluate_king_safety::<White>(pos, &mut score, params);
    evaluate_king_safety::<Black>(pos, &mut score, params);

    taper(score, phase, pos.side_to_move)
}

fn lazy_piece_terms<S: Side>(
    mut bb: Bitboard,
    value: Option<Score>,
    pst: &[Score; 64],
    score: &mut Score,
) {
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
