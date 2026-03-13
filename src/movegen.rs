use crate::{
    bitboard::{Bitboard, Bitboards},
    position::Position,
    types::{Move, MoveFlag, Piece, Square},
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

fn generate_king_moves(pos: &Position, bbs: &Bitboards, moves: &mut MoveList) {
    let colour = pos.side_to_move;
    let us = colour.idx();
    let them = colour.opposite().idx();
    let own_occ = pos.occupancy[us];
    let opp_occ = pos.occupancy[them];

    let mut kings = pos.pieces[us][Piece::King.idx()];
    while !kings.is_empty() {
        let from = kings.pop_lsb();
        // Mask of all quiet/capture moves
        let targets = bbs.king_attacks(from) & !own_occ;

        push_moves(from, targets, opp_occ, moves);
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
    let us = colour.idx();
    let them = colour.opposite().idx();
    let opp_occ = pos.occupancy[them];

    let mut pawns = pos.pieces[us][Piece::Pawn.idx()];
    while !pawns.is_empty() {
        let from = pawns.pop_lsb();

        // Find capture moves
        // Need to consider promotion and en passant
        let mut targets = bbs.pawn_attacks(from, colour) & opp_occ;
        while !targets.is_empty() {
            let to = targets.pop_lsb();
            moves.push(Move::new(from, to, MoveFlag::Capture));
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

        a.sort_by_key(|m| (m.from().idx(), m.to().idx(), m.flag()));
        e.sort_by_key(|m| (m.from().idx(), m.to().idx(), m.flag()));

        assert_eq!(a, e);
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
        generate_knight_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_white);

        moves.clear();
        pos.side_to_move = pos.side_to_move.opposite();
        generate_knight_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_black);

        // Generates correct moves in other position
        let mut pos = Position::from_fen(
            "r1bqkb1r/ppp2ppp/n3pn2/2Pp2B1/3P4/2N2N2/PP2PPPP/R2QKB1R w KQkq - 0 1",
        );
        moves.clear();

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
        generate_knight_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_white);

        moves.clear();
        pos.side_to_move = pos.side_to_move.opposite();
        generate_knight_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_black);
    }

    #[test]
    fn generates_correct_king_moves() {
        let mut pos = Position::default();
        let bbs = Bitboards::init();
        let mut moves = MoveList::new();

        // Default position should yield no king moves
        generate_king_moves(&pos, &bbs, &mut moves);
        assert_eq!(moves.as_slice(), []);

        pos.side_to_move = pos.side_to_move.opposite();
        generate_king_moves(&pos, &bbs, &mut moves);
        assert_eq!(moves.as_slice(), []);

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

        generate_king_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_white);

        moves.clear();
        pos.side_to_move = pos.side_to_move.opposite();
        generate_king_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_black);
    }

    #[test]
    fn generates_correct_pawn_moves() {
        let mut pos = Position::default();
        let bbs = Bitboards::init();
        let mut moves = MoveList::new();

        // No capturing moves in the starting position
        generate_pawn_moves(&pos, &bbs, &mut moves);
        assert_eq!(moves.as_slice(), []);

        pos.side_to_move = pos.side_to_move.opposite();
        generate_pawn_moves(&pos, &bbs, &mut moves);
        assert_eq!(moves.as_slice(), []);

        // Generates correct capturing moves
        let expected_white = [Move::new(Square::C4, Square::D5, MoveFlag::Capture)];
        let expected_black = [
            Move::new(Square::D5, Square::C4, MoveFlag::Capture),
            Move::new(Square::H6, Square::G5, MoveFlag::Capture),
        ];
        let mut pos = Position::from_fen(
            "r1bqk2r/ppp1bpp1/2n1pn1p/3p2B1/2PP4/2N1PN2/PP2BPPP/R2QK2R w KQkq - 0 1",
        );

        generate_pawn_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_white);

        moves.clear();
        pos.side_to_move = pos.side_to_move.opposite();
        generate_pawn_moves(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_black);
    }

    #[test]
    fn generates_correct_pseudo_legal() {
        let mut pos = Position::default();
        let bbs = Bitboards::init();
        let mut moves = MoveList::new();

        // Generates correct moves in starting position
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
        generate_pseudo_legal(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_white);

        moves.clear();
        pos.side_to_move = pos.side_to_move.opposite();
        generate_pseudo_legal(&pos, &bbs, &mut moves);
        assert_same_moves(&moves, &expected_black);
    }
}
