use bitflags::bitflags;
use std::fmt;

use crate::bitboard::Bitboard;

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

    pub const ALL: [Self; 64] = {
        let mut arr = [Self(0); 64];
        let mut i = 0;
        while i < 64 {
            arr[i] = Self(i as u8);
            i += 1;
        }
        arr
    };

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

    #[inline(always)]
    pub const fn is_ok(self) -> bool {
        self.0 >= Self::A1.0 && self.0 <= Self::H8.0
    }

    #[inline(always)]
    pub fn rank_bb(self) -> Bitboard {
        Bitboard::RANK_1 << (self.rank() * 8)
    }

    #[inline(always)]
    pub fn file_bb(self) -> Bitboard {
        Bitboard::FILE_A << self.file()
    }

    #[inline(always)]
    pub fn colour(self) -> Colour {
        match (self.file() + self.rank() + 1) % 2 {
            0 => Colour::White,
            1 => Colour::Black,
            _ => unreachable!(),
        }
    }
}

impl Default for Square {
    fn default() -> Self {
        Self::NONE
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_none() {
            return f.write_str("-");
        }

        let file = (b'a' + self.file()) as char;
        let rank = (b'1' + self.rank()) as char;
        write!(f, "{file}{rank}")
    }
}

// --- Board Info ---
#[repr(i8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Direction {
    North = 8,
    East = 1,
    South = -8,
    West = -1,
    NorthEast = 9,
    NorthWest = 7,
    SouthEast = -7,
    SouthWest = -9,
}

pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = FILE_A << 1;
pub const FILE_C: u64 = FILE_A << 2;
pub const FILE_D: u64 = FILE_A << 3;
pub const FILE_E: u64 = FILE_A << 4;
pub const FILE_F: u64 = FILE_A << 5;
pub const FILE_G: u64 = FILE_A << 6;
pub const FILE_H: u64 = FILE_A << 7;

pub const RANK_1: u64 = 0xFF;
pub const RANK_2: u64 = RANK_1 << 8;
pub const RANK_3: u64 = RANK_1 << 16;
pub const RANK_4: u64 = RANK_1 << 24;
pub const RANK_5: u64 = RANK_1 << 32;
pub const RANK_6: u64 = RANK_1 << 40;
pub const RANK_7: u64 = RANK_1 << 48;
pub const RANK_8: u64 = RANK_1 << 56;

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

    pub fn to_char(self) -> char {
        debug_assert!(!self.is_empty());

        let piece = match self.piece().unwrap() {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        };

        if self.colour().unwrap() == Colour::White {
            piece.to_ascii_uppercase()
        } else {
            piece
        }
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
const CASTLING_MASK: [Castling; 64] = {
    let mut masks: [Castling; 64] = [Castling::DEFAULT; 64];

    masks[0] = Castling::DEFAULT.without(Castling::WHITE_OOO); // a1 - white loses queenside
    masks[7] = Castling::DEFAULT.without(Castling::WHITE_OO); // h1 - white loses kingside
    masks[4] = Castling::DEFAULT.without(Castling::WHITE_CASTLING); // e1 - white loses both

    masks[56] = Castling::DEFAULT.without(Castling::BLACK_OOO); // a8 - black loses queenside
    masks[63] = Castling::DEFAULT.without(Castling::BLACK_OO); // h8 - black loses kingside
    masks[60] = Castling::DEFAULT.without(Castling::BLACK_CASTLING); // e8 - black loses both

    masks
};

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CastlingType {
    Kingside = 0,
    Queenside = 1,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub struct Castling: u8 {
        const NONE = 0;

        const WHITE_OO = 1 << 0;
        const WHITE_OOO = 1 << 1;
        const BLACK_OO = 1 << 2;
        const BLACK_OOO = 1 << 3;

        const KINGSIDE = Self::WHITE_OO.bits() | Self::BLACK_OO.bits();
        const QUEENSIDE = Self::WHITE_OOO.bits() | Self::BLACK_OOO.bits();
        const WHITE_CASTLING = Self::WHITE_OO.bits() | Self::WHITE_OOO.bits();
        const BLACK_CASTLING = Self::BLACK_OO.bits() | Self::BLACK_OOO.bits();
        const DEFAULT = Self::WHITE_CASTLING.bits() | Self::BLACK_CASTLING.bits();
    }
}

impl Castling {
    pub const fn new(bits: u8) -> Self {
        Self::from_bits_retain(bits)
    }

    #[inline(always)]
    pub const fn without(self, other: Self) -> Self {
        Self::from_bits_retain(self.bits() & !other.bits())
    }

    #[inline(always)]
    pub fn update(&mut self, from: Square, to: Square) {
        // AND the rights with the mask of both squares
        // Handles:
        // 1. King moving (loses both)
        // 2. Rook move (loses one)
        // 3. Rook captured (opponent loses one)
        *self &= CASTLING_MASK[from.idx()] & CASTLING_MASK[to.idx()];
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
    pub const fn can_white_ks(self) -> bool {
        self.contains(Castling::WHITE_OO)
    }

    #[inline(always)]
    pub const fn can_white_qs(self) -> bool {
        self.contains(Castling::WHITE_OOO)
    }

    #[inline(always)]
    pub const fn can_black_ks(self) -> bool {
        self.contains(Castling::BLACK_OO)
    }

    #[inline(always)]
    pub const fn can_black_qs(self) -> bool {
        self.contains(Castling::BLACK_OOO)
    }
}

impl Default for Castling {
    #[inline(always)]
    fn default() -> Self {
        Self::DEFAULT
    }
}

// --- Moves ---
bitflags! {
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub struct MoveFlag: u8 {
        const Quiet = 0b0000;
        const DoublePush = 0b0001;
        const KingCastle = 0b0010;
        const QueenCastle = 0b0011;
        const Capture = 0b0100;
        const EpCapture = 0b0101;

        const PromoN = 0b1000;
        const PromoB = 0b1001;
        const PromoR = 0b1010;
        const PromoQ = 0b1011;

        const PromoCaptureN = 0b1100;
        const PromoCaptureB = 0b1101;
        const PromoCaptureR = 0b1110;
        const PromoCaptureQ = 0b1111;
    }
}

impl MoveFlag {
    #[inline(always)]
    pub const fn u16(self) -> u16 {
        self.bits() as u16
    }
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
        Self(from.u16() | to.u16() << 6 | flag.u16() << 12)
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
    pub fn flag(self) -> MoveFlag {
        MoveFlag::from_bits_retain((self.0 >> 12) as u8)
    }

    #[inline(always)]
    pub fn is_capture(self) -> bool {
        self.flag().contains(MoveFlag::Capture)
    }

    #[inline(always)]
    pub fn is_ep_capture(self) -> bool {
        self.flag() == MoveFlag::EpCapture
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
        self.flag().bits() & 0b1000 != 0
    }

    #[inline(always)]
    pub fn promotion_piece(self) -> Option<Piece> {
        if !self.is_promotion() {
            return None;
        }
        // Last two bits determine promotion type
        match self.flag().bits() & 0b0011 {
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
            x if x == MoveFlag::KingCastle => Some(CastlingType::Kingside),
            x if x == MoveFlag::QueenCastle => Some(CastlingType::Queenside),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn is_double_push(self) -> bool {
        self.flag() == MoveFlag::DoublePush
    }

    #[inline(always)]
    pub fn is_quiet(self) -> bool {
        self.flag() == MoveFlag::Quiet
    }

    #[inline(always)]
    pub fn is_null(self) -> bool {
        self == Self::NULL
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut m = format!("{}{}", self.from(), self.to());

        if let Some(piece) = self.promotion_piece() {
            let p = match piece {
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                Piece::Queen => 'q',
                _ => unreachable!(),
            };
            m.push(p);
        };

        write!(f, "{m}")
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

    #[test]
    fn square_colour_is_correct() {
        assert_eq!(Square::A1.colour(), Colour::Black);
        assert_eq!(Square::A8.colour(), Colour::White);
        assert_eq!(Square::H1.colour(), Colour::White);
        assert_eq!(Square::H8.colour(), Colour::Black);
        assert_eq!(Square::E4.colour(), Colour::White);
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
        assert_eq!(pos.castling_rights, Castling::BLACK_CASTLING);
        pos.undo_move(king_move, &state);

        // Moving rook clears castling rights and rook capture clears opposition rights
        let rook_move = Move::new(Square::H1, Square::H8, MoveFlag::Capture);
        pos.make_move(rook_move, &mut state);
        assert_eq!(pos.castling_rights, Castling::QUEENSIDE);
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
        assert_eq!(PAWN_E4.flag(), MoveFlag::DoublePush)
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
