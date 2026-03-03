pub type Square = u8;
pub type Bitboard = u64;

pub const NO_SQUARE: Square = 64;

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

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Colour {
    White = 0,
    Black = 1,
    Both = 2,
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
        self.0 &= CASTLING_MASK[from as usize] & CASTLING_MASK[to as usize];
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
        Self((from as u16) | ((to as u16) << 6) | ((flag as u16) << 12))
    }

    #[inline(always)]
    pub fn from(self) -> Square {
        (self.0 & 0x3F) as Square
    }

    #[inline(always)]
    pub fn to(self) -> Square {
        ((self.0 >> 6) & 0x3F) as Square
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
