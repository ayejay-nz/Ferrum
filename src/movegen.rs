use crate::{
    bitboard::{Bitboard, bitboards},
    position::Position,
    types::{Castling, CastlingType, Colour, Direction, Move, MoveFlag, Piece, Square},
};

pub const MAX_MOVES: usize = 256;

#[derive(Debug)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GenType {
    All,
    Noisy,
    Quiets,
    Evasions,
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

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        &mut self.moves[..self.len]
    }
}

fn get_evasion_mask(pos: &Position) -> Bitboard {
    let bbs = bitboards();
    let us = pos.side_to_move;
    let king_sq = pos.king_square(us);

    debug_assert!(pos.checkers.bit_count() == 1);
    let checker_sq = pos.checkers.lsb();

    bbs.evasion_mask(king_sq, checker_sq)
}

fn push_moves(
    from: Square,
    targets: Bitboard,
    opp_occ: Bitboard,
    moves: &mut MoveList,
    mode: GenType,
) {
    if mode == GenType::All || mode == GenType::Noisy || mode == GenType::Evasions {
        let mut captures = targets & opp_occ;

        while !captures.is_empty() {
            let to = captures.pop_lsb();
            moves.push(Move::new(from, to, MoveFlag::Capture));
        }
    }

    if mode == GenType::All || mode == GenType::Quiets || mode == GenType::Evasions {
        let mut quiets = targets & !opp_occ;

        while !quiets.is_empty() {
            let to = quiets.pop_lsb();
            moves.push(Move::new(from, to, MoveFlag::Quiet));
        }
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

fn generate_king_step_moves(pos: &Position, moves: &mut MoveList, mode: GenType) {
    let bbs = bitboards();

    let us = pos.side_to_move;
    let them = us.opposite();
    let from = pos.king_square(us);
    let own_occ = pos.occupancy[us.idx()];
    let opp_occ = pos.occupancy[them.idx()];

    // Mask of all quiet/capture move squares
    let mut targets = bbs.king_attacks(from) & !own_occ;

    while !targets.is_empty() {
        let to = targets.pop_lsb();

        // Move leaves the king in check
        let occ = (pos.occupancy[2] ^ from.bitboard()) | to.bitboard();
        if pos.attackers_to_exist(to, occ) {
            continue;
        }

        let is_capture = !(opp_occ & to.bitboard()).is_empty();
        match mode {
            GenType::Quiets if !is_capture => moves.push(Move::new(from, to, MoveFlag::Quiet)),
            GenType::Noisy if is_capture => moves.push(Move::new(from, to, MoveFlag::Capture)),
            GenType::All | GenType::Evasions => moves.push(Move::new(
                from,
                to,
                if is_capture {
                    MoveFlag::Capture
                } else {
                    MoveFlag::Quiet
                },
            )),
            _ => {}
        }
    }
    push_moves(from, targets, opp_occ, moves, mode);
}

fn generate_castling_moves(pos: &Position, moves: &mut MoveList) {
    let us = pos.side_to_move;
    let king_square = pos.king_square(us);

    // Find castling moves, if any
    let (ks_right, qs_right) = match us {
        Colour::White => (Castling::WHITE_OO, Castling::WHITE_OOO),
        Colour::Black => (Castling::BLACK_OO, Castling::BLACK_OOO),
        _ => unreachable!(),
    };

    if pos.can_castle(ks_right) && !pos.castling_impeded(ks_right) {
        push_castling(king_square, CastlingType::Kingside, moves);
    }

    if pos.can_castle(qs_right) && !pos.castling_impeded(qs_right) {
        push_castling(king_square, CastlingType::Queenside, moves);
    }
}

fn generate_king_moves(pos: &Position, moves: &mut MoveList, mode: GenType) {
    match mode {
        GenType::All => {
            generate_king_step_moves(pos, moves, GenType::All);
            generate_castling_moves(pos, moves);
        }
        GenType::Noisy => {
            generate_king_step_moves(pos, moves, GenType::Noisy);
        }
        GenType::Quiets => {
            generate_king_step_moves(pos, moves, GenType::Quiets);
            generate_castling_moves(pos, moves);
        }
        GenType::Evasions => {
            generate_king_step_moves(pos, moves, GenType::All);
        }
    }
}

fn generate_knight_moves(
    pos: &Position,
    moves: &mut MoveList,
    mode: GenType,
    evasion_mask: Option<Bitboard>,
) {
    let bbs = bitboards();

    let us = pos.side_to_move;
    let them = us.opposite();
    let own_occ = pos.occupancy[us.idx()];
    let opp_occ = pos.occupancy[them.idx()];

    let mut knights = pos.pieces[us.idx()][Piece::Knight.idx()];

    while !knights.is_empty() {
        let from = knights.pop_lsb();

        // Mask of all quiet/capture moves
        let mut targets = bbs.knight_attacks(from) & !own_occ;
        if let Some(mask) = evasion_mask {
            targets &= mask;
        }
        push_moves(from, targets, opp_occ, moves, mode);
    }
}

fn generate_pawn_pushes(pos: &Position, moves: &mut MoveList, evasion_mask: Option<Bitboard>) {
    let us = pos.side_to_move;
    let is_white = us == Colour::White;
    let all_occ = pos.occupancy[2];
    let empty_squares = !all_occ;

    #[rustfmt::skip]
    let relative_rank_7 = if is_white { Bitboard::RANK_7 } else { Bitboard::RANK_2 };
    #[rustfmt::skip]
    let relative_rank_3 = if is_white { Bitboard::RANK_3 } else { Bitboard::RANK_6 };
    #[rustfmt::skip]
    let up = if is_white { Direction::North } else { Direction::South };

    let pawns = pos.pieces[us.idx()][Piece::Pawn.idx()];
    let pawns_not_on_7 = pawns & !relative_rank_7;

    let mut bb1 = pawns_not_on_7.shift(up) & empty_squares;
    let mut bb2 = (bb1 & relative_rank_3).shift(up) & empty_squares;

    if let Some(mask) = evasion_mask {
        bb1 &= mask;
        bb2 &= mask;
    }

    push_pawn_moves(up as i8, &mut bb1, MoveFlag::Quiet, moves);
    push_pawn_moves(up as i8 + up as i8, &mut bb2, MoveFlag::DoublePush, moves);
}

fn generate_pawn_captures(pos: &Position, moves: &mut MoveList, evasion_mask: Option<Bitboard>) {
    let us = pos.side_to_move;
    let them = us.opposite();
    let is_white = us == Colour::White;
    let opp_occ = pos.occupancy[them.idx()];

    #[rustfmt::skip]
    let relative_rank_7 = if is_white { Bitboard::RANK_7 } else { Bitboard::RANK_2 };
    #[rustfmt::skip]
    let up_left = if is_white { Direction::NorthWest } else { Direction::SouthEast };
    #[rustfmt::skip]
    let up_right = if is_white { Direction::NorthEast } else { Direction::SouthWest };

    let pawns = pos.pieces[us.idx()][Piece::Pawn.idx()];
    let pawns_not_on_7 = pawns & !relative_rank_7;

    let mut bb1 = pawns_not_on_7.shift(up_left) & opp_occ;
    let mut bb2 = pawns_not_on_7.shift(up_right) & opp_occ;

    if let Some(mask) = evasion_mask {
        bb1 &= mask;
        bb2 &= mask;
    }

    push_pawn_moves(up_left as i8, &mut bb1, MoveFlag::Capture, moves);
    push_pawn_moves(up_right as i8, &mut bb2, MoveFlag::Capture, moves);
}

fn generate_pawn_promotions(pos: &Position, moves: &mut MoveList, evasion_mask: Option<Bitboard>) {
    let us = pos.side_to_move;
    let them = us.opposite();
    let is_white = us == Colour::White;
    let opp_occ = pos.occupancy[them.idx()];
    let all_occ = pos.occupancy[2];
    let empty_squares = !all_occ;

    #[rustfmt::skip]
    let relative_rank_7 = if is_white { Bitboard::RANK_7 } else { Bitboard::RANK_2 };
    #[rustfmt::skip]
    let up = if is_white { Direction::North } else { Direction::South };
    #[rustfmt::skip]
    let up_left = if is_white { Direction::NorthWest } else { Direction::SouthEast };
    #[rustfmt::skip]
    let up_right = if is_white { Direction::NorthEast } else { Direction::SouthWest };

    let pawns = pos.pieces[us.idx()][Piece::Pawn.idx()];
    let pawns_on_7 = pawns & relative_rank_7;

    if !pawns_on_7.is_empty() {
        let mut bb1 = pawns_on_7.shift(up) & empty_squares;
        let mut bb2 = pawns_on_7.shift(up_left) & opp_occ;
        let mut bb3 = pawns_on_7.shift(up_right) & opp_occ;

        if let Some(mask) = evasion_mask {
            bb1 &= mask;
            bb2 &= mask;
            bb3 &= mask;
        }

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
}

fn generate_en_passant(pos: &Position, moves: &mut MoveList) {
    let bbs = bitboards();

    let us = pos.side_to_move;
    let them = us.opposite();
    let is_white = us == Colour::White;

    #[rustfmt::skip]
    let relative_rank_5 = if is_white { Bitboard::RANK_5 } else { Bitboard::RANK_4};

    let pawns = pos.pieces[us.idx()][Piece::Pawn.idx()];
    let pawns_on_5 = pawns & relative_rank_5;

    if !pos.ep_square.is_none() {
        let mut bb1 = pawns_on_5 & bbs.pawn_attacks(pos.ep_square, them);

        while !bb1.is_empty() {
            moves.push(Move::new(bb1.pop_lsb(), pos.ep_square, MoveFlag::EpCapture));
        }
    }
}

fn generate_pawn_moves(
    pos: &Position,
    moves: &mut MoveList,
    mode: GenType,
    evasion_mask: Option<Bitboard>,
) {
    match mode {
        GenType::All => {
            generate_pawn_pushes(pos, moves, evasion_mask);
            generate_pawn_captures(pos, moves, evasion_mask);
            generate_pawn_promotions(pos, moves, evasion_mask);
            generate_en_passant(pos, moves);
        }
        GenType::Noisy => {
            generate_pawn_captures(pos, moves, evasion_mask);
            generate_en_passant(pos, moves);
            generate_pawn_promotions(pos, moves, evasion_mask);
        }
        GenType::Quiets => {
            generate_pawn_pushes(pos, moves, evasion_mask);
        }
        GenType::Evasions => {
            generate_pawn_captures(pos, moves, evasion_mask);
            generate_pawn_pushes(pos, moves, evasion_mask);
            generate_pawn_promotions(pos, moves, evasion_mask);

            // En passant is an annoying case. Since it is so rare,
            // we just generate it and test it later for legality
            generate_en_passant(pos, moves);
        }
    }
}

fn generate_bishop_slider_moves(
    pos: &Position,
    moves: &mut MoveList,
    mode: GenType,
    evasion_mask: Option<Bitboard>,
) {
    let bbs = bitboards();

    let colour = pos.side_to_move;
    let us = colour.idx();
    let them = colour.opposite().idx();
    let own_occ = pos.occupancy[us];
    let opp_occ = pos.occupancy[them];
    let all_occ = pos.occupancy[2];

    let mut bishop_sliders =
        pos.pieces[us][Piece::Bishop.idx()] | pos.pieces[us][Piece::Queen.idx()];

    while !bishop_sliders.is_empty() {
        let from = bishop_sliders.pop_lsb();

        // Mask of all quiet/capture moves
        let mut targets = bbs.bishop_attacks(from, all_occ) & !own_occ;
        if let Some(mask) = evasion_mask {
            targets &= mask;
        }
        push_moves(from, targets, opp_occ, moves, mode);
    }
}

fn generate_rook_slider_moves(
    pos: &Position,
    moves: &mut MoveList,
    mode: GenType,
    evasion_mask: Option<Bitboard>,
) {
    let bbs = bitboards();

    let colour = pos.side_to_move;
    let us = colour.idx();
    let them = colour.opposite().idx();
    let own_occ = pos.occupancy[us];
    let opp_occ = pos.occupancy[them];
    let all_occ = pos.occupancy[2];

    let mut rook_sliders = pos.pieces[us][Piece::Rook.idx()] | pos.pieces[us][Piece::Queen.idx()];

    while !rook_sliders.is_empty() {
        let from = rook_sliders.pop_lsb();

        // Mask of all quiet/capture moves
        let mut targets = bbs.rook_attacks(from, all_occ) & !own_occ;
        if let Some(mask) = evasion_mask {
            targets &= mask;
        }
        push_moves(from, targets, opp_occ, moves, mode);
    }
}

pub fn generate_all(pos: &Position, moves: &mut MoveList) {
    debug_assert!(pos.checkers.is_empty());
    moves.clear();

    generate_pawn_moves(pos, moves, GenType::All, None);
    generate_knight_moves(pos, moves, GenType::All, None);
    generate_bishop_slider_moves(pos, moves, GenType::All, None);
    generate_rook_slider_moves(pos, moves, GenType::All, None);
    generate_king_moves(pos, moves, GenType::All);
}

pub fn generate_noisy(pos: &Position, moves: &mut MoveList) {
    debug_assert!(pos.checkers.is_empty());
    moves.clear();

    generate_pawn_moves(pos, moves, GenType::Noisy, None);
    generate_knight_moves(pos, moves, GenType::Noisy, None);
    generate_bishop_slider_moves(pos, moves, GenType::Noisy, None);
    generate_rook_slider_moves(pos, moves, GenType::Noisy, None);
    generate_king_moves(pos, moves, GenType::Noisy);
}

pub fn generate_legal_noisy(pos: &Position, moves: &mut MoveList) -> MoveList {
    debug_assert!(pos.checkers.is_empty());

    let mut legal_moves = MoveList::new();
    generate_noisy(pos, moves);
    filter_moves(pos, moves, &mut legal_moves);

    legal_moves
}

pub fn generate_quiets(pos: &Position, moves: &mut MoveList) {
    debug_assert!(pos.checkers.is_empty());
    moves.clear();

    generate_pawn_moves(pos, moves, GenType::Quiets, None);
    generate_knight_moves(pos, moves, GenType::Quiets, None);
    generate_bishop_slider_moves(pos, moves, GenType::Quiets, None);
    generate_rook_slider_moves(pos, moves, GenType::Quiets, None);
    generate_king_moves(pos, moves, GenType::Quiets)
}

pub fn generate_evasions(pos: &Position, moves: &mut MoveList) {
    moves.clear();

    let checkers = pos.checkers;
    debug_assert!(checkers.bit_count() > 0);

    // Skip non-king move generation if double check
    if checkers.bit_count() == 1 {
        let evasion_mask = Some(get_evasion_mask(pos));
        generate_pawn_moves(pos, moves, GenType::Evasions, evasion_mask);
        generate_knight_moves(pos, moves, GenType::Evasions, evasion_mask);
        generate_bishop_slider_moves(pos, moves, GenType::Evasions, evasion_mask);
        generate_rook_slider_moves(pos, moves, GenType::Evasions, evasion_mask);
    }

    generate_king_moves(pos, moves, GenType::Evasions);
}

pub fn generate(mode: GenType, pos: &Position, moves: &mut MoveList) {
    match mode {
        GenType::All => generate_all(pos, moves),
        GenType::Noisy => generate_noisy(pos, moves),
        GenType::Quiets => generate_quiets(pos, moves),
        GenType::Evasions => generate_evasions(pos, moves),
    }
}

fn filter_moves(pos: &Position, moves: &mut MoveList, legal_moves: &mut MoveList) {
    let us = pos.side_to_move;
    let pinned = pos.pinned_pieces(us) & pos.occupancy[us.idx()];

    for &mv in moves.as_slice() {
        // Only need to check if a move is legal if it moves
        // a pinned piece, is a castling move, or is en passant
        if !(pinned & mv.from().bitboard()).is_empty() || mv.is_castle() || mv.is_ep_capture() {
            if pos.is_legal(mv) {
                legal_moves.push(mv);
            }
        } else {
            legal_moves.push(mv);
        }
    }
}

pub fn generate_legal(pos: &Position, moves: &mut MoveList) -> MoveList {
    let mut legal_moves = MoveList::new();

    if pos.checkers.is_empty() {
        generate(GenType::All, pos, moves);
    } else {
        generate(GenType::Evasions, pos, moves);
    }

    filter_moves(pos, moves, &mut legal_moves);

    legal_moves
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
        expected_white: &[Move],
        expected_black: &[Move],
        piece: Piece,
        moves: &mut MoveList,
    ) {
        if piece == Piece::King {
            generate_king_moves(pos, moves, GenType::All);
        } else {
            let generate_moves = match piece {
                Piece::Pawn => generate_pawn_moves,
                Piece::Knight => generate_knight_moves,
                Piece::Bishop => generate_bishop_slider_moves,
                Piece::Rook => generate_rook_slider_moves,
                _ => unreachable!(),
            };
            generate_moves(pos, moves, GenType::All, None);
        };

        assert_same_moves(moves, expected_white);

        moves.clear();
        pos.side_to_move = pos.side_to_move.opposite();

        if piece == Piece::King {
            generate_king_moves(pos, moves, GenType::All);
        } else {
            let generate_moves = match piece {
                Piece::Pawn => generate_pawn_moves,
                Piece::Knight => generate_knight_moves,
                Piece::Bishop => generate_bishop_slider_moves,
                Piece::Rook => generate_rook_slider_moves,
                _ => unreachable!(),
            };
            generate_moves(pos, moves, GenType::All, None);
        };

        assert_same_moves(moves, expected_black);

        moves.clear();
    }

    #[test]
    fn generates_correct_knight_moves() {
        let mut pos = Position::default();
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
            &expected_white,
            &expected_black,
            Piece::Knight,
            &mut moves,
        );
    }

    #[test]
    fn generates_correct_king_moves() {
        let mut pos = Position::default();
        let mut moves = MoveList::new();

        // Default position should yield no king moves
        assert_both_sides(&mut pos, &[], &[], Piece::King, &mut moves);

        // Gets correct king moves in other position
        let expected_white = [
            Move::new(Square::D6, Square::E6, MoveFlag::Capture),
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
            &expected_white,
            &expected_black,
            Piece::King,
            &mut moves,
        );
    }

    #[test]
    fn generates_correct_pawn_moves() {
        let mut pos = Position::default();
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
        generate_pawn_moves(&pos, &mut moves, GenType::All, None);
        assert_same_moves(&moves, &expected_white);
        moves.clear();

        let expected_black = [
            Move::new(Square::C4, Square::C3, MoveFlag::Quiet),
            Move::new(Square::E4, Square::E3, MoveFlag::Quiet),
            Move::new(Square::C4, Square::D3, MoveFlag::EpCapture),
            Move::new(Square::E4, Square::D3, MoveFlag::EpCapture),
        ];
        let pos = Position::from_fen("3k4/8/8/8/2pPp3/8/8/3K4 b - d3 0 1");
        generate_pawn_moves(&pos, &mut moves, GenType::All, None);
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
            &expected_white,
            &expected_black,
            Piece::Pawn,
            &mut moves,
        );
    }

    #[test]
    fn generates_correct_bishop_sliders() {
        let mut pos = Position::default();
        let mut moves = MoveList::new();

        // No moves in the default position
        let expected_white = [];
        let expected_black = [];
        assert_both_sides(
            &mut pos,
            &expected_white,
            &expected_black,
            Piece::Bishop,
            &mut moves,
        );

        let expected_white = [
            Move::new(Square::C2, Square::B1, MoveFlag::Quiet),
            Move::new(Square::C2, Square::B3, MoveFlag::Quiet),
            Move::new(Square::C2, Square::D3, MoveFlag::Quiet),
            Move::new(Square::C2, Square::E4, MoveFlag::Capture),
            Move::new(Square::D1, Square::E2, MoveFlag::Quiet),
            Move::new(Square::D1, Square::F3, MoveFlag::Quiet),
            Move::new(Square::D1, Square::G4, MoveFlag::Capture),
        ];
        let expected_black = [
            Move::new(Square::D8, Square::C7, MoveFlag::Capture),
            Move::new(Square::D8, Square::E7, MoveFlag::Quiet),
            Move::new(Square::G4, Square::H5, MoveFlag::Quiet),
            Move::new(Square::G4, Square::H3, MoveFlag::Quiet),
            Move::new(Square::G4, Square::F3, MoveFlag::Quiet),
            Move::new(Square::G4, Square::E2, MoveFlag::Quiet),
            Move::new(Square::G4, Square::D1, MoveFlag::Capture),
        ];
        let mut pos = Position::from_fen("2rqk3/2N5/5p2/3p1n2/N2Pp1b1/6n1/2B5/3QK3 w - - 0 1");
        assert_both_sides(
            &mut pos,
            &expected_white,
            &expected_black,
            Piece::Bishop,
            &mut moves,
        );
    }

    #[test]
    fn generates_correct_rook_sliders() {
        let mut pos = Position::default();
        let mut moves = MoveList::new();

        // No moves in the default position
        let expected_white = [];
        let expected_black = [];
        assert_both_sides(
            &mut pos,
            &expected_white,
            &expected_black,
            Piece::Rook,
            &mut moves,
        );

        let expected_white = [
            Move::new(Square::B1, Square::C1, MoveFlag::Quiet),
            Move::new(Square::B1, Square::D1, MoveFlag::Quiet),
            Move::new(Square::B1, Square::B2, MoveFlag::Quiet),
            Move::new(Square::B1, Square::B3, MoveFlag::Quiet),
            Move::new(Square::B1, Square::B4, MoveFlag::Capture),
            Move::new(Square::D2, Square::D1, MoveFlag::Quiet),
            Move::new(Square::D2, Square::E2, MoveFlag::Quiet),
            Move::new(Square::D2, Square::F2, MoveFlag::Quiet),
            Move::new(Square::D2, Square::C2, MoveFlag::Capture),
            Move::new(Square::D2, Square::D3, MoveFlag::Quiet),
            Move::new(Square::D2, Square::D4, MoveFlag::Capture),
        ];
        let expected_black = [
            Move::new(Square::B4, Square::C4, MoveFlag::Quiet),
            Move::new(Square::B4, Square::B3, MoveFlag::Quiet),
            Move::new(Square::B4, Square::B2, MoveFlag::Quiet),
            Move::new(Square::B4, Square::B1, MoveFlag::Capture),
            Move::new(Square::D4, Square::C4, MoveFlag::Quiet),
            Move::new(Square::D4, Square::E4, MoveFlag::Quiet),
            Move::new(Square::D4, Square::F4, MoveFlag::Capture),
            Move::new(Square::D4, Square::D3, MoveFlag::Quiet),
            Move::new(Square::D4, Square::D2, MoveFlag::Capture),
        ];
        let mut pos = Position::from_fen("4k3/8/8/1n1p4/pr1q1N2/8/2bQ2B1/BR2K3 w - - 0 1");
        assert_both_sides(
            &mut pos,
            &expected_white,
            &expected_black,
            Piece::Rook,
            &mut moves,
        );
    }

    #[test]
    fn generates_correct_pseudo_legal() {
        let mut pos = Position::default();
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

        generate_all(&pos, &mut moves);
        assert_same_moves(&moves, &expected_white);

        moves.clear();
        pos.side_to_move = pos.side_to_move.opposite();
        generate_all(&pos, &mut moves);
        assert_same_moves(&moves, &expected_black);

        moves.clear();
    }
}
