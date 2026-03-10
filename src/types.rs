// --- Squares ---
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Square(u8);

macro_rules! define_squares {
    ($($name:ident = $val:expr),* $(,)?) => {
        $(
            pub const $name: Square = Square($val);
        )*
    };
}

impl Square {
    pub const NONE: Self = Self(64);

    // Define all square values using macro
    define_squares! {
        A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63,
        A7 = 48, B7 = 49, C7 = 50, D7 = 51, E7 = 52, F7 = 53, G7 = 54, H7 = 55,
        A6 = 40, B6 = 41, C6 = 42, D6 = 43, E6 = 44, F6 = 45, G6 = 46, H6 = 47,
        A5 = 32, B5 = 33, C5 = 34, D5 = 35, E5 = 36, F5 = 37, G5 = 38, H5 = 39,
        A4 = 24, B4 = 25, C4 = 26, D4 = 27, E4 = 28, F4 = 29, G4 = 30, H4 = 31,
        A3 = 16, B3 = 17, C3 = 18, D3 = 19, E3 = 20, F3 = 21, G3 = 22, H3 = 23,
        A2 = 8,  B2 = 9,  C2 = 10, D2 = 11, E2 = 12, F2 = 13, G2 = 14, H2 = 15,
        A1 = 0,  B1 = 1,  C1 = 2,  D1 = 3,  E1 = 4,  F1 = 5,  G1 = 6,  H1 = 7,
    }

    #[inline(always)]
    pub const fn new(sq: u8) -> Self {
        Self(sq)
    }

    #[inline(always)]
    pub const fn rank(self) -> u8 {
        self.0 >> 3
    }

    #[inline(always)]
    pub const fn file(self) -> u8 {
        self.0 & 7
    }

    #[inline(always)]
    pub const fn from_coords(rank: u8, file: u8) -> Self {
        // (rank * 8) + file
        Self((rank << 3) | (file & 7))
    }

    #[inline(always)]
    pub const fn is_none(self) -> bool {
        self.0 == Self::NONE.0
    }

    #[inline(always)]
    pub const fn u8(self) -> u8 {
        self.0 as u8
    }

    #[inline(always)]
    pub const fn u16(self) -> u16 {
        self.0 as u16
    }

    #[inline(always)]
    pub const fn idx(self) -> usize {
        self.0 as usize
    }

    #[inline(always)]
    pub const fn bit(self) -> u64 {
        1u64 << self.0
    }

    #[inline(always)]
    pub const fn bitboard(self) -> Bitboard {
        Bitboard::new(self.bit())
    }
}

impl Default for Square {
    fn default() -> Self {
        Self::NONE
    }
}

// --- Bitboards ---
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Bitboard(u64);

impl Bitboard {
    #[inline(always)]
    pub const fn new(bb: u64) -> Self {
        Self(bb)
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn set_square(&mut self, square: Square) {
        self.0 |= 1 << square.u8()
    }

    #[inline(always)]
    pub const fn clear_square(&mut self, square: Square) {
        self.0 &= !(1 << square.u8())
    }

    #[inline(always)]
    pub const fn u64(&self) -> u64 {
        self.0 as u64
    }

    #[inline(always)]
    pub fn lsb(self) -> Square {
        debug_assert!(self.0 != 0);
        Square::new(self.0.trailing_zeros() as u8)
    }

    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Square {
        debug_assert!(self.0 != 0);
        let sq = self.lsb();
        self.0 &= self.0 - 1;
        sq
    }

    #[inline(always)]
    pub const fn lsb_bb(self) -> u64 {
        self.0 & self.0.wrapping_neg()
    }
}

// --- Pieces ---
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Colour {
    White = 0,
    Black = 1,
}

impl Colour {
    #[inline(always)]
    pub const fn idx(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn opposite(self) -> Colour {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl Piece {
    pub const fn idx(self) -> usize {
        self as usize
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PieceCode(u8);

impl PieceCode {
    pub const EMPTY: Self = Self(12);

    // Use bit 1-3 for piece type, bit 0 for (inverted) colour, i.e. 0 = B, 1 = W
    // This sets the PieceCode according to the polyglot book format encoding
    pub const fn new(colour: Colour, piece: Piece) -> Self {
        Self((piece as u8) << 1 | ((colour as u8) ^ 0b1))
    }

    #[inline(always)]
    pub const fn idx(self) -> usize {
        self.0 as usize
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }

    #[inline(always)]
    pub const fn colour(self) -> Option<Colour> {
        if self.is_empty() {
            return None;
        }
        // Bit 0 is the colour, 0 = black, 1 = white
        if (self.0 & 0b0001) == 1 {
            Some(Colour::White)
        } else {
            Some(Colour::Black)
        }
    }

    #[inline(always)]
    pub const fn piece(self) -> Option<Piece> {
        if self.is_empty() {
            return None;
        }
        // Bits 1-3 determine piece type
        match (self.0 >> 1) & 0b0111 {
            0 => Some(Piece::Pawn),
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            5 => Some(Piece::King),
            _ => unreachable!(),
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        let colour = if c.is_uppercase() {
            Colour::White
        } else {
            Colour::Black
        };
        let piece = match c.to_ascii_lowercase() {
            'p' => Piece::Pawn,
            'n' => Piece::Knight,
            'b' => Piece::Bishop,
            'r' => Piece::Rook,
            'q' => Piece::Queen,
            'k' => Piece::King,
            _ => return None,
        };

        return Some(PieceCode::new(colour, piece));
    }
}

// --- Mailbox ---
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Mailbox([PieceCode; 64]);

impl Mailbox {
    #[inline(always)]
    pub const fn new() -> Self {
        Self([PieceCode::EMPTY; 64])
    }

    #[inline(always)]
    pub const fn set_square(&mut self, square: Square, pc: PieceCode) {
        self.0[square.idx()] = pc
    }

    #[inline(always)]
    pub const fn clear_square(&mut self, square: Square) {
        self.0[square.idx()] = PieceCode::EMPTY
    }

    #[inline(always)]
    pub const fn piece_code_at(&self, square: Square) -> PieceCode {
        self.0[square.idx()]
    }

    #[inline(always)]
    pub const fn piece_at(&self, square: Square) -> Option<Piece> {
        self.piece_code_at(square).piece()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Square, PieceCode)> + '_ {
        self.0
            .iter()
            .copied()
            .enumerate()
            .map(|(i, pc)| (Square::new(i as u8), pc))
    }

    pub fn iter_occupied(&self) -> impl Iterator<Item = (Square, PieceCode)> + '_ {
        self.iter().filter(|(_, pc)| !pc.is_empty())
    }
}

// --- Castling ---
const CASTLING_MASK: [u8; 64] = {
    let mut masks: [u8; 64] = [0xF; 64];

    masks[0] = !Castling::WQ_BIT; // a1 - white loses WQ
    masks[7] = !Castling::WK_BIT; // h1 - white loses WK
    masks[4] = !(Castling::WK_BIT | Castling::WQ_BIT); // e1 - white loses BOTH

    masks[56] = !Castling::BQ_BIT; // a8 - black loses BQ
    masks[63] = !Castling::BK_BIT; // h8 - black loses BK
    masks[60] = !(Castling::BK_BIT | Castling::BQ_BIT); // e8 - black loses BOTH

    masks
};

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CastlingType {
    Kingside = 0,
    Queenside = 1,
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Castling(u8);

impl Castling {
    pub const WK_BIT: u8 = 1 << 0;
    pub const WQ_BIT: u8 = 1 << 1;
    pub const BK_BIT: u8 = 1 << 2;
    pub const BQ_BIT: u8 = 1 << 3;

    pub const WK: Self = Self(Self::WK_BIT);
    pub const WQ: Self = Self(Self::WQ_BIT);
    pub const BK: Self = Self(Self::BK_BIT);
    pub const BQ: Self = Self(Self::BQ_BIT);

    pub const NONE: Self = Self(0);
    pub const DEFAULT: Self = Self(Self::WK_BIT | Self::WQ_BIT | Self::BK_BIT | Self::BQ_BIT);

    pub const fn new(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub fn update(&mut self, from: Square, to: Square) {
        // AND the rights with the mask of both squares
        // Handles:
        // 1. King moving (loses both)
        // 2. Rook move (loses one)
        // 3. Rook captured (opponent loses one)
        self.0 &= CASTLING_MASK[from.idx()] & CASTLING_MASK[to.idx()];
    }

    #[inline(always)]
    pub fn get_rook_squares_from_castle(colour: Colour, side: CastlingType) -> (Square, Square) {
        // Return a tuple of the rooks move as (from, to) squares
        match (colour, side) {
            (Colour::White, CastlingType::Kingside) => (Square::H1, Square::F1),
            (Colour::White, CastlingType::Queenside) => (Square::A1, Square::D1),
            (Colour::Black, CastlingType::Kingside) => (Square::H8, Square::F8),
            (Colour::Black, CastlingType::Queenside) => (Square::A8, Square::D8),
        }
    }

    #[inline(always)]
    pub fn can_white_ks(self) -> bool {
        (self.0 & Self::WK_BIT) != 0
    }

    #[inline(always)]
    pub fn can_white_qs(self) -> bool {
        (self.0 & Self::WQ_BIT) != 0
    }

    #[inline(always)]
    pub fn can_black_ks(self) -> bool {
        (self.0 & Self::BK_BIT) != 0
    }

    #[inline(always)]
    pub fn can_black_qs(self) -> bool {
        (self.0 & Self::BQ_BIT) != 0
    }
}

// --- Moves ---
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MoveFlag {
    Quiet = 0b0000,
    DoublePush = 0b0001,
    KingCastle = 0b0010,
    QueenCastle = 0b0011,
    Capture = 0b0100,
    EpCapture = 0b0101,

    PromoN = 0b1000,
    PromoB = 0b1001,
    PromoR = 0b1010,
    PromoQ = 0b1011,

    PromoCaptureN = 0b1100,
    PromoCaptureB = 0b1101,
    PromoCaptureR = 0b1110,
    PromoCaptureQ = 0b1111,
}

// Bits 0-5: source square
// Bits 6-11: destination square
// Bits 12-15: Flags
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Move(u16);

impl Move {
    pub const NULL: Self = Self(0);

    #[inline(always)]
    pub const fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        Self(from.u16() | to.u16() << 6 | (flag as u16) << 12)
    }

    #[inline(always)]
    pub fn from(self) -> Square {
        Square::new((self.0 & 0x3F) as u8)
    }

    #[inline(always)]
    pub fn to(self) -> Square {
        Square::new(((self.0 >> 6) & 0x3F) as u8)
    }

    #[inline(always)]
    pub fn flag(self) -> u8 {
        (self.0 >> 12) as u8
    }

    #[inline(always)]
    pub fn is_capture(self) -> bool {
        (self.flag() & 0b0100) != 0
    }

    #[inline(always)]
    pub fn is_ep_capture(self) -> bool {
        self.flag() == 0b0101
    }

    #[inline(always)]
    pub fn get_ep_pawn_square(self) -> Square {
        // Ep capture will only happen on rank 5 for white and rank 2 for black
        match self.to().rank() {
            5 => Square::new(self.to().u8() - 8),
            2 => Square::new(self.to().u8() + 8),
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn is_promotion(self) -> bool {
        (self.flag() & 0b1000) != 0
    }

    #[inline(always)]
    pub fn promotion_piece(self) -> Option<Piece> {
        if !self.is_promotion() {
            return None;
        }
        // Last two bits determine promotion type
        match self.flag() & 0b0011 {
            0 => Some(Piece::Knight),
            1 => Some(Piece::Bishop),
            2 => Some(Piece::Rook),
            3 => Some(Piece::Queen),
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn is_castle(self) -> bool {
        self.castle_type().is_some()
    }

    #[inline(always)]
    pub fn castle_type(self) -> Option<CastlingType> {
        match self.flag() {
            x if x == MoveFlag::KingCastle as u8 => Some(CastlingType::Kingside),
            x if x == MoveFlag::QueenCastle as u8 => Some(CastlingType::Queenside),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_double_push(self) -> bool {
        self.flag() == 0b0001
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::position::{Position, StateInfo};

    const PAWN_E4: Move = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);

    fn set_board(fen: &str) -> (Position, StateInfo) {
        let pos = Position::from_fen(fen);
        let mut state = StateInfo::new();
        state.set_from_position(&pos);
        return (pos, state);
    }

    // --- Squares ---
    #[test]
    fn rank_returns_expected_rank() {
        assert_eq!(Square::A1.rank(), 0);
        assert_eq!(Square::A8.rank(), 7);
        assert_eq!(Square::E4.rank(), 3);
        assert_eq!(Square::H1.rank(), 0);
        assert_eq!(Square::H8.rank(), 7);
    }

    #[test]
    fn file_returns_expected_file() {
        assert_eq!(Square::A1.file(), 0);
        assert_eq!(Square::A8.file(), 0);
        assert_eq!(Square::E4.file(), 4);
        assert_eq!(Square::H1.file(), 7);
        assert_eq!(Square::H8.file(), 7);
    }

    #[test]
    fn square_from_coords_is_correct() {
        assert_eq!(Square::from_coords(0, 0), Square::A1);
        assert_eq!(Square::from_coords(7, 0), Square::A8);
        assert_eq!(Square::from_coords(3, 4), Square::E4);
        assert_eq!(Square::from_coords(0, 7), Square::H1);
        assert_eq!(Square::from_coords(7, 7), Square::H8);
    }

    #[test]
    fn square_is_none() {
        assert!(Square::is_none(Square::NONE));
        assert_eq!(Square::H8.is_none(), false);
    }

    #[test]
    fn square_bit_is_correct() {
        assert_eq!(Square::A1.bit(), 1);
        assert_eq!(Square::A8.bit(), 2u64.pow(56));
        assert_eq!(Square::E4.bit(), 2u64.pow(28));
        assert_eq!(Square::H1.bit(), 2u64.pow(7));
        assert_eq!(Square::H8.bit(), 2u64.pow(63));
    }

    // --- Bitboards ---
    #[test]
    fn bitboard_updates_work() {
        let mut bb = Bitboard::new(0);
        let mut bb_value = 0;
        bb.set_square(Square::E4);
        bb_value += 1 << Square::E4.u8();
        assert_eq!(bb.0, bb_value);
        bb.set_square(Square::H8);
        bb_value += 1 << Square::H8.u8();
        assert_eq!(bb.0, bb_value);

        bb.clear_square(Square::E4);
        bb_value -= 1 << Square::E4.u8();
        assert_eq!(bb.0, bb_value);
        bb.clear_square(Square::H8);
        bb_value -= 1 << Square::H8.u8();
        assert_eq!(bb.0, bb_value);
    }

    #[test]
    #[should_panic(expected = "assertion failed: self.0 != 0")]
    fn bitboard_lsb_is_correct() {
        let (pos, _) = set_board("8/1B6/5k2/1P5p/6r1/2NK4/8/8 w - - 0 1");
        assert_eq!(pos.occupancy[Colour::White.idx()].lsb(), Square::C3);
        assert_eq!(pos.occupancy[Colour::Black.idx()].lsb(), Square::G4);
        assert_eq!(pos.occupancy[2].lsb(), Square::C3);

        // Should panic
        Bitboard::new(0).lsb();
    }

    #[test]
    #[should_panic(expected = "assertion failed: self.0 != 0")]
    fn bitboard_pop_lsb_is_correct() {
        let (mut pos, _) = set_board("8/1B6/5k2/1P5p/6r1/2NK4/8/8 w - - 0 1");
        let (start, _) = set_board("8/1B6/5k2/1P5p/6r1/2NK4/8/8 w - - 0 1");

        assert_eq!(pos.occupancy[Colour::White.idx()].pop_lsb(), Square::C3);
        assert_ne!(pos.occupancy[Colour::White.idx()], start.occupancy[Colour::White.idx()]);

        assert_eq!(pos.occupancy[Colour::Black.idx()].pop_lsb(), Square::G4);
        assert_eq!(pos.occupancy[Colour::Black.idx()].pop_lsb(), Square::H5);
        assert_ne!(pos.occupancy[Colour::Black.idx()], start.occupancy[Colour::Black.idx()]);

        // Should panic
        Bitboard::new(0).pop_lsb();
    }

    #[test]
    fn bitboard_lsb_bb_is_correct() {
        let (pos, _) = set_board("8/1B6/5k2/1P5p/6r1/2NK4/8/8 w - - 0 1");
        assert_eq!(pos.occupancy[Colour::White.idx()].lsb_bb(), Square::C3.bit());
        assert_eq!(pos.occupancy[Colour::Black.idx()].lsb_bb(), Square::G4.bit());
        assert_eq!(pos.occupancy[2].lsb_bb(), Square::C3.bit());
        assert_eq!(Bitboard::new(0).lsb_bb(), 0u64);
    }

    // --- Pieces ---
    #[test]
    fn piece_code_is_empty() {
        assert!(PieceCode::EMPTY.is_empty());
        assert_eq!(PieceCode::new(Colour::White, Piece::Pawn).is_empty(), false);
    }

    #[test]
    fn piece_code_colour_is_correct() {
        assert_eq!(PieceCode::EMPTY.colour(), None);
        let white_pawn = PieceCode::new(Colour::White, Piece::Pawn);
        let black_pawn = PieceCode::new(Colour::Black, Piece::Pawn);
        assert_eq!(white_pawn.colour(), Some(Colour::White));
        assert_eq!(black_pawn.colour(), Some(Colour::Black));
    }

    #[test]
    fn piece_code_piece_is_correct() {
        assert_eq!(PieceCode::EMPTY.piece(), None);
        let white_pawn = PieceCode::new(Colour::White, Piece::Pawn);
        let white_king = PieceCode::new(Colour::White, Piece::King);
        assert_eq!(white_pawn.piece(), Some(Piece::Pawn));
        assert_eq!(white_king.piece(), Some(Piece::King));
    }

    #[test]
    fn piece_code_from_char_is_correct() {
        assert_eq!(PieceCode::from_char('Z'), None);
        let white_pawn = PieceCode::from_char('P');
        let black_king = PieceCode::from_char('k');
        assert_eq!(white_pawn, Some(PieceCode::new(Colour::White, Piece::Pawn)));
        assert_eq!(black_king, Some(PieceCode::new(Colour::Black, Piece::King)));
    }

    // --- Mailbox ---
    #[test]
    fn mailbox_piece_code_at_is_correct() {
        let mut mailbox = Mailbox::new();
        let white_pawn = PieceCode::new(Colour::White, Piece::Pawn);
        mailbox.set_square(Square::E4, white_pawn);
        assert_eq!(mailbox.piece_code_at(Square::E4), white_pawn);
    }

    // --- Castling ---
    #[test]
    fn castling_update_is_correct() {
        let (mut pos, mut state) = set_board("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");

        // Moving king clears castling rights
        let king_move = Move::new(Square::E1, Square::F1, MoveFlag::Quiet);
        pos.make_move(king_move, &mut state);
        assert_eq!(
            pos.castling_rights.bits(),
            Castling::BK_BIT | Castling::BQ_BIT
        );
        pos.undo_move(king_move, &state);

        // Moving rook clears castling rights and rook capture clears opposition rights
        let rook_move = Move::new(Square::H1, Square::H8, MoveFlag::Capture);
        pos.make_move(rook_move, &mut state);
        assert_eq!(
            pos.castling_rights.bits(),
            Castling::BQ_BIT | Castling::WQ_BIT
        );
        pos.undo_move(rook_move, &state);
    }

    #[test]
    fn castling_get_rook_squares_is_correct() {
        let (rook_start, rook_end) =
            Castling::get_rook_squares_from_castle(Colour::White, CastlingType::Kingside);
        assert_eq!(rook_start, Square::H1);
        assert_eq!(rook_end, Square::F1);

        let (rook_start, rook_end) =
            Castling::get_rook_squares_from_castle(Colour::Black, CastlingType::Queenside);
        assert_eq!(rook_start, Square::A8);
        assert_eq!(rook_end, Square::D8);
    }

    #[test]
    fn castling_can_castle_is_correct() {
        let default = Castling::DEFAULT;
        assert!(default.can_white_ks());
        assert!(default.can_white_qs());
        assert!(default.can_black_ks());
        assert!(default.can_black_qs());
    }

    // --- Moves ---
    #[test]
    fn move_from_is_correct() {
        assert_eq!(PAWN_E4.from(), Square::E2)
    }

    #[test]
    fn move_to_is_correct() {
        assert_eq!(PAWN_E4.to(), Square::E4)
    }

    #[test]
    fn move_flag_is_correct() {
        assert_eq!(PAWN_E4.flag(), MoveFlag::DoublePush as u8)
    }

    #[test]
    fn move_get_ep_pawn_square_is_correct() {
        let white_ep_move = Move::new(Square::D5, Square::E6, MoveFlag::EpCapture);
        let black_ep_move = Move::new(Square::E4, Square::D3, MoveFlag::EpCapture);
        assert_eq!(white_ep_move.get_ep_pawn_square(), Square::E5);
        assert_eq!(black_ep_move.get_ep_pawn_square(), Square::D4);
    }

    #[test]
    fn move_promotion_piece_is_correct() {
        let non_promo_move = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);
        let promo_move = Move::new(Square::A7, Square::A8, MoveFlag::PromoN);
        let capture_promo_move = Move::new(Square::A7, Square::B8, MoveFlag::PromoCaptureQ);
        assert_eq!(non_promo_move.promotion_piece(), None);
        assert_eq!(promo_move.promotion_piece(), Some(Piece::Knight));
        assert_eq!(capture_promo_move.promotion_piece(), Some(Piece::Queen));
    }

    #[test]
    fn move_castle_type_is_correct() {
        let non_castle_move = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);
        let ks_castle_move = Move::new(Square::E1, Square::G1, MoveFlag::KingCastle);
        let qs_castle_move = Move::new(Square::E1, Square::C1, MoveFlag::QueenCastle);
        assert_eq!(non_castle_move.castle_type(), None);
        assert_eq!(ks_castle_move.castle_type(), Some(CastlingType::Kingside));
        assert_eq!(qs_castle_move.castle_type(), Some(CastlingType::Queenside));
    }
}
