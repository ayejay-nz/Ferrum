use crate::{
    bitboard::{Bitboard, Bitboards},
    position::Position,
    types::{Castling, CastlingType, Colour, Direction, Move, MoveFlag, Piece, Square},
};

pub const MAX_MOVES: usize = 256;

#[derive(Debug)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl MoveList {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; MAX_MOVES],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0
    }

    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        debug_assert!(self.len < MAX_MOVES);
        self.moves[self.len] = mv;
        self.len += 1;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }
}

fn push_moves(from: Square, targets: Bitboard, opp_occ: Bitboard, moves: &mut MoveList) {
    let mut captures = targets & opp_occ;
    let mut quiets = targets ^ captures;

    while !captures.is_empty() {
        let to = captures.pop_lsb();
        moves.push(Move::new(from, to, MoveFlag::Capture));
    }

    while !quiets.is_empty() {
        let to = quiets.pop_lsb();
        moves.push(Move::new(from, to, MoveFlag::Quiet));
    }
}

fn push_pawn_moves(offset: i8, targets: &mut Bitboard, flag: MoveFlag, moves: &mut MoveList) {
    while !targets.is_empty() {
        let to = targets.pop_lsb();
        let from = Square::new((to.u8() as i8 - offset) as u8);
        moves.push(Move::new(from, to, flag));
    }
}

fn push_promotions(offset: i8, to: Square, is_capture: bool, moves: &mut MoveList) {
    let flag_mask = if is_capture {
        MoveFlag::Capture
    } else {
        MoveFlag::Quiet
    };
    let from = Square::new((to.u8() as i8 - offset) as u8);

    moves.push(Move::new(from, to, MoveFlag::PromoQ | flag_mask));
    moves.push(Move::new(from, to, MoveFlag::PromoN | flag_mask));
    moves.push(Move::new(from, to, MoveFlag::PromoR | flag_mask));
    moves.push(Move::new(from, to, MoveFlag::PromoB | flag_mask));
}

fn push_castling(king_square: Square, ct: CastlingType, moves: &mut MoveList) {
    let mv = match ct {
        CastlingType::Kingside => Move::new(
            king_square,
            Square::new(king_square.u8() + 2),
            MoveFlag::KingCastle,
        ),
        CastlingType::Queenside => Move::new(
            king_square,
            Square::new(king_square.u8() - 2),
            MoveFlag::QueenCastle,
        ),
    };
    moves.push(mv);
}

fn generate_king_moves(pos: &Position, bbs: &Bitboards, moves: &mut MoveList) {
    let colour = pos.side_to_move;
    let is_white = colour == Colour::White;
    let us = colour.idx();
    let them = colour.opposite().idx();
    let own_occ = pos.occupancy[us];
    let opp_occ = pos.occupancy[them];

    #[rustfmt::skip]
    let king_square = if is_white { pos.white_king_square } else { pos.black_king_square };
    // Mask of all quiet/capture move squares
    let targets = bbs.king_attacks(king_square) & !own_occ;
    push_moves(king_square, targets, opp_occ, moves);

    // Find castling moves, if any
    let (ks_right, qs_right) = match colour {
        Colour::White => (Castling::WHITE_OO, Castling::WHITE_OOO),
        Colour::Black => (Castling::BLACK_OO, Castling::BLACK_OOO),
    };

    if pos.can_castle(ks_right) && !pos.castling_impeded(ks_right) {
        push_castling(king_square, CastlingType::Kingside, moves);
    }

    if pos.can_castle(qs_right) && !pos.castling_impeded(qs_right) {
        push_castling(king_square, CastlingType::Queenside, moves);
    }
}

fn generate_knight_moves(pos: &Position, bbs: &Bitboards, moves: &mut MoveList) {
    let colour = pos.side_to_move;
    let us = colour.idx();
    let them = colour.opposite().idx();
    let own_occ = pos.occupancy[us];
    let opp_occ = pos.occupancy[them];

    let mut knights = pos.pieces[us][Piece::Knight.idx()];
    while !knights.is_empty() {
        let from = knights.pop_lsb();
        // Mask of all quiet/capture moves
        let targets = bbs.knight_attacks(from) & !own_occ;

        push_moves(from, targets, opp_occ, moves);
    }
}

fn generate_pawn_moves(pos: &Position, bbs: &Bitboards, moves: &mut MoveList) {
    let colour = pos.side_to_move;
    let is_white = colour == Colour::White;
    let us = colour.idx();
    let them = colour.opposite().idx();
    let opp_occ = pos.occupancy[them];
    let all_occ = pos.occupancy[2];
    let empty_squares = !all_occ;

    #[rustfmt::skip]
    let relative_rank_7 = if is_white { Bitboard::RANK_7 } else { Bitboard::RANK_2 };
    #[rustfmt::skip]
    let relative_rank_3 = if is_white { Bitboard::RANK_3 } else { Bitboard::RANK_6 };
    #[rustfmt::skip]
    let up = if is_white { Direction::North } else { Direction::South };
    #[rustfmt::skip]
    let up_left = if is_white { Direction::NorthWest } else { Direction::SouthEast };
    #[rustfmt::skip]
    let up_right = if is_white { Direction::NorthEast } else { Direction::SouthWest };

    let pawns = pos.pieces[us][Piece::Pawn.idx()];
    let pawns_on_7 = pawns & relative_rank_7;
    let pawns_not_on_7 = pawns ^ pawns_on_7;

    // Consider only regular single and double pushes
    let mut bb1 = pawns_not_on_7.shift(up) & empty_squares;
    let mut bb2 = (bb1 & relative_rank_3).shift(up) & empty_squares;

    push_pawn_moves(up as i8, &mut bb1, MoveFlag::Quiet, moves);
    push_pawn_moves(up as i8 + up as i8, &mut bb2, MoveFlag::DoublePush, moves);

    // Promotions
    if !pawns_on_7.is_empty() {
        let mut bb1 = pawns_on_7.shift(up) & empty_squares;
        let mut bb2 = pawns_on_7.shift(up_left) & opp_occ;
        let mut bb3 = pawns_on_7.shift(up_right) & opp_occ;

        while !bb1.is_empty() {
            push_promotions(up as i8, bb1.pop_lsb(), false, moves);
        }
        while !bb2.is_empty() {
            push_promotions(up_left as i8, bb2.pop_lsb(), true, moves);
        }
        while !bb3.is_empty() {
            push_promotions(up_right as i8, bb3.pop_lsb(), true, moves);
        }
    }

    // Regular captures and en passant
    let mut bb1 = pawns_not_on_7.shift(up_left) & opp_occ;
    let mut bb2 = pawns_not_on_7.shift(up_right) & opp_occ;

    push_pawn_moves(up_left as i8, &mut bb1, MoveFlag::Capture, moves);
    push_pawn_moves(up_right as i8, &mut bb2, MoveFlag::Capture, moves);

    if !pos.ep_square.is_none() {
        bb1 = pawns_not_on_7 & bbs.pawn_attacks(pos.ep_square, colour.opposite());

        while !bb1.is_empty() {
            moves.push(Move::new(bb1.pop_lsb(), pos.ep_square, MoveFlag::EpCapture));
        }
    }
}

pub fn generate_pseudo_legal(pos: &Position, bbs: &Bitboards, moves: &mut MoveList) {
    moves.clear();

    generate_pawn_moves(pos, bbs, moves);
    generate_knight_moves(pos, bbs, moves);
    generate_king_moves(pos, bbs, moves);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_same_moves(actual: &MoveList, expected: &[Move]) {
        let mut a = actual.as_slice().to_vec();
        let mut e = expected.to_vec();

        a.sort_by_key(|m| (m.from().idx(), m.to().idx(), m.flag().bits()));
        e.sort_by_key(|m| (m.from().idx(), m.to().idx(), m.flag().bits()));

        assert_eq!(a, e);
    }

    fn assert_both_sides(
        pos: &mut Position,
        bbs: &Bitboards,
        expected_white: &[Move],
        expected_black: &[Move],
        piece: Piece,
        moves: &mut MoveList,
    ) {
        let generate_moves = match piece {
            Piece::Pawn => generate_pawn_moves,
            Piece::Knight => generate_knight_moves,
            Piece::King => generate_king_moves,
            _ => unreachable!(),
        };

        generate_moves(pos, bbs, moves);
        assert_same_moves(moves, expected_white);

        moves.clear();
        pos.side_to_move = pos.side_to_move.opposite();
        generate_moves(pos, bbs, moves);
        assert_same_moves(moves, expected_black);

        moves.clear();
    }

    #[test]
    fn generates_correct_knight_moves() {
        let mut pos = Position::default();
        let bbs = Bitboards::init();
        let mut moves = MoveList::new();

        // Generates the correct moves in the starting position as white and black
        let expected_white = [
            Move::new(Square::B1, Square::A3, MoveFlag::Quiet),
            Move::new(Square::B1, Square::C3, MoveFlag::Quiet),
            Move::new(Square::G1, Square::F3, MoveFlag::Quiet),
            Move::new(Square::G1, Square::H3, MoveFlag::Quiet),
        ];
        let expected_black = [
            Move::new(Square::B8, Square::A6, MoveFlag::Quiet),
            Move::new(Square::B8, Square::C6, MoveFlag::Quiet),
            Move::new(Square::G8, Square::F6, MoveFlag::Quiet),
            Move::new(Square::G8, Square::H6, MoveFlag::Quiet),
        ];
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::Knight,
            &mut moves,
        );

        // Generates correct moves in other position
        let mut pos = Position::from_fen(
            "r1bqkb1r/ppp2ppp/n3pn2/2Pp2B1/3P4/2N2N2/PP2PPPP/R2QKB1R w KQkq - 0 1",
        );

        let expected_white = [
            Move::new(Square::C3, Square::D5, MoveFlag::Capture),
            Move::new(Square::C3, Square::B5, MoveFlag::Quiet),
            Move::new(Square::C3, Square::A4, MoveFlag::Quiet),
            Move::new(Square::C3, Square::B1, MoveFlag::Quiet),
            Move::new(Square::C3, Square::E4, MoveFlag::Quiet),
            Move::new(Square::F3, Square::E5, MoveFlag::Quiet),
            Move::new(Square::F3, Square::H4, MoveFlag::Quiet),
            Move::new(Square::F3, Square::G1, MoveFlag::Quiet),
            Move::new(Square::F3, Square::D2, MoveFlag::Quiet),
        ];
        let expected_black = [
            Move::new(Square::A6, Square::B4, MoveFlag::Quiet),
            Move::new(Square::A6, Square::B8, MoveFlag::Quiet),
            Move::new(Square::A6, Square::C5, MoveFlag::Capture),
            Move::new(Square::F6, Square::G8, MoveFlag::Quiet),
            Move::new(Square::F6, Square::H5, MoveFlag::Quiet),
            Move::new(Square::F6, Square::G4, MoveFlag::Quiet),
            Move::new(Square::F6, Square::E4, MoveFlag::Quiet),
            Move::new(Square::F6, Square::D7, MoveFlag::Quiet),
        ];
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::Knight,
            &mut moves,
        );
    }

    #[test]
    fn generates_correct_king_moves() {
        let mut pos = Position::default();
        let bbs = Bitboards::init();
        let mut moves = MoveList::new();

        // Default position should yield no king moves
        assert_both_sides(&mut pos, &bbs, &[], &[], Piece::King, &mut moves);

        // Gets correct king moves in other position
        let expected_white = [
            Move::new(Square::D6, Square::C7, MoveFlag::Quiet),
            Move::new(Square::D6, Square::D7, MoveFlag::Quiet),
            Move::new(Square::D6, Square::E7, MoveFlag::Quiet),
            Move::new(Square::D6, Square::E6, MoveFlag::Capture),
            Move::new(Square::D6, Square::D5, MoveFlag::Quiet),
            Move::new(Square::D6, Square::C5, MoveFlag::Quiet),
            Move::new(Square::D6, Square::C6, MoveFlag::Quiet),
        ];
        let expected_black = [
            Move::new(Square::A8, Square::B8, MoveFlag::Quiet),
            Move::new(Square::A8, Square::B7, MoveFlag::Capture),
            Move::new(Square::A8, Square::A7, MoveFlag::Quiet),
        ];
        let mut pos = Position::from_fen("k7/1N3rp1/3Kp2p/4P2P/8/6B1/8/8 w - - 0 1");
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::King,
            &mut moves,
        );

        // Correctly generates castling moves
        let expected_white = [
            Move::new(Square::E1, Square::G1, MoveFlag::KingCastle),
            Move::new(Square::E1, Square::C1, MoveFlag::QueenCastle),
            Move::new(Square::E1, Square::F1, MoveFlag::Quiet),
            Move::new(Square::E1, Square::D1, MoveFlag::Quiet),
        ];
        let expected_black = [
            Move::new(Square::E8, Square::G8, MoveFlag::KingCastle),
            Move::new(Square::E8, Square::C8, MoveFlag::QueenCastle),
            Move::new(Square::E8, Square::F8, MoveFlag::Quiet),
            Move::new(Square::E8, Square::D8, MoveFlag::Quiet),
        ];
        let mut pos = Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::King,
            &mut moves,
        );

        let expected_white = [
            Move::new(Square::E1, Square::C1, MoveFlag::QueenCastle),
            Move::new(Square::E1, Square::F1, MoveFlag::Quiet),
            Move::new(Square::E1, Square::D1, MoveFlag::Quiet),
        ];
        let expected_black = [
            Move::new(Square::E8, Square::C8, MoveFlag::QueenCastle),
            Move::new(Square::E8, Square::D8, MoveFlag::Quiet),
        ];
        let mut pos = Position::from_fen("r3kb1r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w Qkq - 0 1");
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::King,
            &mut moves,
        );
    }

    #[test]
    fn generates_correct_pawn_moves() {
        let mut pos = Position::default();
        let bbs = Bitboards::init();
        let mut moves = MoveList::new();

        // Generates correct pawn moves in starting position
        let expected_white = [
            Move::new(Square::A2, Square::A3, MoveFlag::Quiet),
            Move::new(Square::A2, Square::A4, MoveFlag::DoublePush),
            Move::new(Square::B2, Square::B3, MoveFlag::Quiet),
            Move::new(Square::B2, Square::B4, MoveFlag::DoublePush),
            Move::new(Square::C2, Square::C3, MoveFlag::Quiet),
            Move::new(Square::C2, Square::C4, MoveFlag::DoublePush),
            Move::new(Square::D2, Square::D3, MoveFlag::Quiet),
            Move::new(Square::D2, Square::D4, MoveFlag::DoublePush),
            Move::new(Square::E2, Square::E3, MoveFlag::Quiet),
            Move::new(Square::E2, Square::E4, MoveFlag::DoublePush),
            Move::new(Square::F2, Square::F3, MoveFlag::Quiet),
            Move::new(Square::F2, Square::F4, MoveFlag::DoublePush),
            Move::new(Square::G2, Square::G3, MoveFlag::Quiet),
            Move::new(Square::G2, Square::G4, MoveFlag::DoublePush),
            Move::new(Square::H2, Square::H3, MoveFlag::Quiet),
            Move::new(Square::H2, Square::H4, MoveFlag::DoublePush),
        ];
        let expected_black = [
            Move::new(Square::A7, Square::A6, MoveFlag::Quiet),
            Move::new(Square::A7, Square::A5, MoveFlag::DoublePush),
            Move::new(Square::B7, Square::B6, MoveFlag::Quiet),
            Move::new(Square::B7, Square::B5, MoveFlag::DoublePush),
            Move::new(Square::C7, Square::C6, MoveFlag::Quiet),
            Move::new(Square::C7, Square::C5, MoveFlag::DoublePush),
            Move::new(Square::D7, Square::D6, MoveFlag::Quiet),
            Move::new(Square::D7, Square::D5, MoveFlag::DoublePush),
            Move::new(Square::E7, Square::E6, MoveFlag::Quiet),
            Move::new(Square::E7, Square::E5, MoveFlag::DoublePush),
            Move::new(Square::F7, Square::F6, MoveFlag::Quiet),
            Move::new(Square::F7, Square::F5, MoveFlag::DoublePush),
            Move::new(Square::G7, Square::G6, MoveFlag::Quiet),
            Move::new(Square::G7, Square::G5, MoveFlag::DoublePush),
            Move::new(Square::H7, Square::H6, MoveFlag::Quiet),
            Move::new(Square::H7, Square::H5, MoveFlag::DoublePush),
        ];
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::Pawn,
            &mut moves,
        );

        // Correctly finds double moves
        let expected_white = [
            Move::new(Square::A2, Square::A3, MoveFlag::Quiet),
            Move::new(Square::A2, Square::A4, MoveFlag::DoublePush),
            Move::new(Square::H3, Square::H4, MoveFlag::Quiet),
        ];
        let expected_black = [
            Move::new(Square::H7, Square::H6, MoveFlag::Quiet),
            Move::new(Square::H7, Square::H5, MoveFlag::DoublePush),
            Move::new(Square::A6, Square::A5, MoveFlag::Quiet),
        ];
        let mut pos = Position::from_fen("4k3/7p/p7/8/8/7P/P7/4K3 w - - 0 1");
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::Pawn,
            &mut moves,
        );

        // Correctly finds capture moves
        let expected_white = [
            Move::new(Square::C4, Square::D5, MoveFlag::Capture),
            Move::new(Square::C4, Square::C5, MoveFlag::Quiet),
            Move::new(Square::H3, Square::G4, MoveFlag::Capture),
        ];
        let expected_black = [
            Move::new(Square::D5, Square::C4, MoveFlag::Capture),
            Move::new(Square::D5, Square::D4, MoveFlag::Quiet),
            Move::new(Square::H4, Square::G3, MoveFlag::Capture),
        ];
        let mut pos = Position::from_fen("4k3/8/8/rN1p4/P1P1p1np/n3P1BP/8/4K3 w - - 0 1");
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::Pawn,
            &mut moves,
        );

        // Correctly finds en passant captures
        let expected_white = [
            Move::new(Square::C5, Square::C6, MoveFlag::Quiet),
            Move::new(Square::E5, Square::E6, MoveFlag::Quiet),
            Move::new(Square::C5, Square::D6, MoveFlag::EpCapture),
            Move::new(Square::E5, Square::D6, MoveFlag::EpCapture),
        ];
        let pos = Position::from_fen("3k4/8/8/2PpP3/8/8/8/3K4 w - d6 0 1");
        generate_pawn_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_white);
        moves.clear();

        let expected_black = [
            Move::new(Square::C4, Square::C3, MoveFlag::Quiet),
            Move::new(Square::E4, Square::E3, MoveFlag::Quiet),
            Move::new(Square::C4, Square::D3, MoveFlag::EpCapture),
            Move::new(Square::E4, Square::D3, MoveFlag::EpCapture),
        ];
        let pos = Position::from_fen("3k4/8/8/8/2pPp3/8/8/3K4 b - d3 0 1");
        generate_pawn_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_black);
        moves.clear();

        // Correctly finds promotion moves
        let expected_white = [
            Move::new(Square::G7, Square::G8, MoveFlag::PromoN),
            Move::new(Square::G7, Square::G8, MoveFlag::PromoB),
            Move::new(Square::G7, Square::G8, MoveFlag::PromoR),
            Move::new(Square::G7, Square::G8, MoveFlag::PromoQ),
        ];
        let expected_black = [
            Move::new(Square::A2, Square::A1, MoveFlag::PromoN),
            Move::new(Square::A2, Square::A1, MoveFlag::PromoB),
            Move::new(Square::A2, Square::A1, MoveFlag::PromoR),
            Move::new(Square::A2, Square::A1, MoveFlag::PromoQ),
        ];
        let mut pos = Position::from_fen("3k4/6P1/8/8/8/8/p7/3K4 w - - 0 1");
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::Pawn,
            &mut moves,
        );

        // Correctly finds capturing promotions
        let expected_white = [
            Move::new(Square::H7, Square::G8, MoveFlag::PromoCaptureN),
            Move::new(Square::H7, Square::G8, MoveFlag::PromoCaptureB),
            Move::new(Square::H7, Square::G8, MoveFlag::PromoCaptureR),
            Move::new(Square::H7, Square::G8, MoveFlag::PromoCaptureQ),
        ];
        let expected_black = [
            Move::new(Square::G2, Square::F1, MoveFlag::PromoCaptureN),
            Move::new(Square::G2, Square::F1, MoveFlag::PromoCaptureB),
            Move::new(Square::G2, Square::F1, MoveFlag::PromoCaptureR),
            Move::new(Square::G2, Square::F1, MoveFlag::PromoCaptureQ),
        ];
        let mut pos = Position::from_fen("3k2nr/7P/8/8/8/8/6p1/3K1BN1 w - - 0 1");
        assert_both_sides(
            &mut pos,
            &bbs,
            &expected_white,
            &expected_black,
            Piece::Pawn,
            &mut moves,
        );
    }

    #[test]
    fn generates_correct_pseudo_legal() {
        let mut pos = Position::default();
        let bbs = Bitboards::init();
        let mut moves = MoveList::new();

        // Generates correct moves in starting position
        let expected_white = [
            Move::new(Square::A2, Square::A3, MoveFlag::Quiet),
            Move::new(Square::A2, Square::A4, MoveFlag::DoublePush),
            Move::new(Square::B2, Square::B3, MoveFlag::Quiet),
            Move::new(Square::B2, Square::B4, MoveFlag::DoublePush),
            Move::new(Square::C2, Square::C3, MoveFlag::Quiet),
            Move::new(Square::C2, Square::C4, MoveFlag::DoublePush),
            Move::new(Square::D2, Square::D3, MoveFlag::Quiet),
            Move::new(Square::D2, Square::D4, MoveFlag::DoublePush),
            Move::new(Square::E2, Square::E3, MoveFlag::Quiet),
            Move::new(Square::E2, Square::E4, MoveFlag::DoublePush),
            Move::new(Square::F2, Square::F3, MoveFlag::Quiet),
            Move::new(Square::F2, Square::F4, MoveFlag::DoublePush),
            Move::new(Square::G2, Square::G3, MoveFlag::Quiet),
            Move::new(Square::G2, Square::G4, MoveFlag::DoublePush),
            Move::new(Square::H2, Square::H3, MoveFlag::Quiet),
            Move::new(Square::H2, Square::H4, MoveFlag::DoublePush),
            Move::new(Square::B1, Square::A3, MoveFlag::Quiet),
            Move::new(Square::B1, Square::C3, MoveFlag::Quiet),
            Move::new(Square::G1, Square::F3, MoveFlag::Quiet),
            Move::new(Square::G1, Square::H3, MoveFlag::Quiet),
        ];
        let expected_black = [
            Move::new(Square::A7, Square::A6, MoveFlag::Quiet),
            Move::new(Square::A7, Square::A5, MoveFlag::DoublePush),
            Move::new(Square::B7, Square::B6, MoveFlag::Quiet),
            Move::new(Square::B7, Square::B5, MoveFlag::DoublePush),
            Move::new(Square::C7, Square::C6, MoveFlag::Quiet),
            Move::new(Square::C7, Square::C5, MoveFlag::DoublePush),
            Move::new(Square::D7, Square::D6, MoveFlag::Quiet),
            Move::new(Square::D7, Square::D5, MoveFlag::DoublePush),
            Move::new(Square::E7, Square::E6, MoveFlag::Quiet),
            Move::new(Square::E7, Square::E5, MoveFlag::DoublePush),
            Move::new(Square::F7, Square::F6, MoveFlag::Quiet),
            Move::new(Square::F7, Square::F5, MoveFlag::DoublePush),
            Move::new(Square::G7, Square::G6, MoveFlag::Quiet),
            Move::new(Square::G7, Square::G5, MoveFlag::DoublePush),
            Move::new(Square::H7, Square::H6, MoveFlag::Quiet),
            Move::new(Square::H7, Square::H5, MoveFlag::DoublePush),
            Move::new(Square::B8, Square::A6, MoveFlag::Quiet),
            Move::new(Square::B8, Square::C6, MoveFlag::Quiet),
            Move::new(Square::G8, Square::F6, MoveFlag::Quiet),
            Move::new(Square::G8, Square::H6, MoveFlag::Quiet),
        ];

        generate_pseudo_legal(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_white);

        moves.clear();
        pos.side_to_move = pos.side_to_move.opposite();
        generate_pseudo_legal(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_black);

        moves.clear();
    }
}
