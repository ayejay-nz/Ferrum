pub type Square = u8;

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
