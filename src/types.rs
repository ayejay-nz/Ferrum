pub type Bitboard = u64;

// --- Squares ---
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Square(u8);

impl Square {
    pub const NONE: Self = Self(64);

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
}

impl Default for Square {
    fn default() -> Self {
        Self::NONE
    }
}

// --- Pieces ---
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Colour {
    White = 0,
    Black = 1,
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

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PieceCode(u8);

impl PieceCode {
    pub const EMPTY: Self = Self(12);

    // Use bit 0-2 for piece type, bit 3 for colour
    pub const fn new(colour: Colour, piece: Piece) -> Self {
        Self((piece as u8) | ((colour as u8) << 3))
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
        // Bit 3 is the colour, 0 = white, 1 = black
        if (self.0 & 0b1000) == 0 {
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
        // First 3 bits determine piece type
        match self.0 & 0b111 {
            0 => Some(Piece::Pawn),
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            5 => Some(Piece::King),
            _ => unreachable!(),
        }
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
    pub fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
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
        // Check if bits are 0010 or 0011
        (self.flag() & 0b1110) == 0b0010
    }
}
