use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

use crate::types::{self, Colour, Direction, Square};

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Bitboard(u64);

impl Bitboard {
    const FILE_A: Bitboard = Bitboard(types::FILE_A);
    const FILE_H: Bitboard = Bitboard(types::FILE_H);

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

    #[inline(always)]
    pub fn shift(self, d: Direction) -> Self {
        match d {
            Direction::North => self << 8,
            Direction::South => self >> 8,
            Direction::East => (self & !Self::FILE_H) << 1,
            Direction::West => (self & !Self::FILE_A) >> 1,

            Direction::NorthEast => (self & !Self::FILE_H) << 9,
            Direction::NorthWest => (self & !Self::FILE_A) << 7,
            Direction::SouthEast => (self & !Self::FILE_H) >> 7,
            Direction::SouthWest => (self & !Self::FILE_A) >> 9,
        }
    }
}

// bb & bb
impl BitAnd for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitAndAssign for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

// bb | bb
impl BitOr for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

// bb ^ bb
impl BitXor for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl BitXorAssign for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

// !bb
impl Not for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        Self(!self.0)
    }
}

// bb << n
impl Shl<u8> for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn shl(self, rhs: u8) -> Self {
        Self(self.0 << rhs)
    }
}

// bb >> n
impl Shr<u8> for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn shr(self, rhs: u8) -> Self {
        Self(self.0 >> rhs)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Bitboards {
    evasion_masks: [[Bitboard; 64]; 64],
    knight_attacks: [Bitboard; 64],
    king_attacks: [Bitboard; 64],
    pawn_attacks: [[Bitboard; 64]; 2],
}

impl Bitboards {
    fn populate_path(s1: Square, s2: Square, bb_table: &mut [[Bitboard; 64]; 64]) {
        let mut step = if s2.rank() > s1.rank() {
            8
        } else if s2.rank() < s1.rank() {
            -8
        } else {
            0
        };
        step += if s2.file() > s1.file() {
            1
        } else if s2.file() < s1.file() {
            -1
        } else {
            0
        };

        let mut sq = s1.u8() as i32 + step;
        let end = s2.u8() as i32;

        while sq != end {
            bb_table[s1.idx()][s2.idx()].set_square(Square::new(sq as u8));
            sq += step;
        }
    }

    /// Returns the bitboard of the target square for the given step, or if
    /// the target square is off the board, returns an empty bitboard
    fn safe_destination(sq: Square, step: i32) -> Bitboard {
        let to = Square::new((sq.u8() as i32 + step) as u8);
        return if to.is_ok() && sq.file().abs_diff(to.file()) <= 2 {
            to.bitboard()
        } else {
            Bitboard::new(0)
        };
    }

    fn knight_attack(sq: Square) -> Bitboard {
        let mut b = Bitboard::new(0);
        for step in [-17, -15, -10, -6, 6, 10, 15, 17] {
            b |= Self::safe_destination(sq, step);
        }
        b
    }

    fn king_attack(sq: Square) -> Bitboard {
        let mut b = Bitboard::new(0);
        for step in [-9, -8, -7, -1, 1, 7, 8, 9] {
            b |= Self::safe_destination(sq, step);
        }
        b
    }

    fn pawn_attack(bb: Bitboard, colour: Colour) -> Bitboard {
        match colour {
            Colour::White => bb.shift(Direction::NorthEast) | bb.shift(Direction::NorthWest),
            Colour::Black => bb.shift(Direction::SouthEast) | bb.shift(Direction::SouthWest),
        }
    }

    /// Initialise the various bitboard tables
    pub fn init() -> Self {
        let mut evasion_masks = [[Bitboard::new(0); 64]; 64];
        let mut knight_attacks = [Bitboard::new(0); 64];
        let mut king_attacks = [Bitboard::new(0); 64];
        let mut pawn_attacks = [[Bitboard::new(0); 64]; 2];

        for s1 in Square::ALL {
            for s2 in Square::ALL {
                if s1 == s2 {
                    continue;
                }

                let same_rank = s1.rank() == s2.rank();
                let same_file = s1.file() == s2.file();
                let same_diag = s1.rank().abs_diff(s2.rank()) == s1.file().abs_diff(s2.file());

                if same_rank || same_file || same_diag {
                    Self::populate_path(s1, s2, &mut evasion_masks);
                }

                // Include destination square always for check evasion (knights)
                evasion_masks[s1.idx()][s2.idx()].set_square(s2);
            }
        }

        // Generate attacking masks for knight, king, and pawns
        for sq in Square::ALL {
            knight_attacks[sq.idx()] = Self::knight_attack(sq);
            king_attacks[sq.idx()] = Self::king_attack(sq);

            for colour in [Colour::White, Colour::Black] {
                pawn_attacks[colour.idx()][sq.idx()] = Self::pawn_attack(sq.bitboard(), colour);
            }
        }

        Self {
            evasion_masks,
            knight_attacks,
            king_attacks,
            pawn_attacks,
        }
    }

    /// Returns a bitboard representing the squares semi-open segment between the two squares
    /// s1 and s2 (exc. s1 but inc. s2). If the two squares are not on the same file/rank/diagonal,
    /// it returns s2. This allows us to generate non-king evasion moves faster.
    pub fn evasion_mask(&self, s1: Square, s2: Square) -> Bitboard {
        return self.evasion_masks[s1.idx()][s2.idx()];
    }

    #[inline(always)]
    pub fn king_attacks(&self, sq: Square) -> Bitboard {
        self.king_attacks[sq.idx()]
    }

    #[inline(always)]
    pub fn knight_attacks(&self, sq: Square) -> Bitboard {
        self.knight_attacks[sq.idx()]
    }

    #[inline(always)]
    pub fn pawn_attacks(&self, sq: Square, colour: Colour) -> Bitboard {
        self.pawn_attacks[colour.idx()][sq.idx()]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{
        position::{Position, StateInfo},
        types::Colour,
    };

    // --- Bitboard ---
    fn set_board(fen: &str) -> (Position, StateInfo) {
        let pos = Position::from_fen(fen);
        let mut state = StateInfo::new();
        state.set_from_position(&pos);
        return (pos, state);
    }

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
    fn bitboard_lsb_is_correct() {
        let (pos, _) = set_board("8/1B6/5k2/1P5p/6r1/2NK4/8/8 w - - 0 1");
        assert_eq!(pos.occupancy[Colour::White.idx()].lsb(), Square::C3);
        assert_eq!(pos.occupancy[Colour::Black.idx()].lsb(), Square::G4);
        assert_eq!(pos.occupancy[2].lsb(), Square::C3);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "assertion failed: self.0 != 0")]
    fn bitboard_lsb_panics_on_empty() {
        Bitboard::new(0).lsb();
    }

    #[test]
    fn bitboard_pop_lsb_is_correct() {
        let (mut pos, _) = set_board("8/1B6/5k2/1P5p/6r1/2NK4/8/8 w - - 0 1");
        let (start, _) = set_board("8/1B6/5k2/1P5p/6r1/2NK4/8/8 w - - 0 1");

        assert_eq!(pos.occupancy[Colour::White.idx()].pop_lsb(), Square::C3);
        assert_ne!(
            pos.occupancy[Colour::White.idx()],
            start.occupancy[Colour::White.idx()]
        );

        assert_eq!(pos.occupancy[Colour::Black.idx()].pop_lsb(), Square::G4);
        assert_eq!(pos.occupancy[Colour::Black.idx()].pop_lsb(), Square::H5);
        assert_ne!(
            pos.occupancy[Colour::Black.idx()],
            start.occupancy[Colour::Black.idx()]
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "assertion failed: self.0 != 0")]
    fn bitboard_pop_lsb_panics_on_empty() {
        Bitboard::new(0).pop_lsb();
    }

    #[test]
    fn bitboard_lsb_bb_is_correct() {
        let (pos, _) = set_board("8/1B6/5k2/1P5p/6r1/2NK4/8/8 w - - 0 1");
        assert_eq!(
            pos.occupancy[Colour::White.idx()].lsb_bb(),
            Square::C3.bit()
        );
        assert_eq!(
            pos.occupancy[Colour::Black.idx()].lsb_bb(),
            Square::G4.bit()
        );
        assert_eq!(pos.occupancy[2].lsb_bb(), Square::C3.bit());
        assert_eq!(Bitboard::new(0).lsb_bb(), 0u64);
    }

    #[test]
    fn bitboard_ops_basic() {
        let a = Bitboard::new(0x0F0F_0000_0000_F0F0);
        let b = Bitboard::new(0x00FF_00FF_00FF_00FF);

        assert_eq!((a & b).u64(), 0x000F_0000_0000_00F0);
        assert_eq!((a | b).u64(), 0x0FFF_00FF_00FF_F0FF);
        assert_eq!((a ^ b).u64(), 0x0FF0_00FF_00FF_F00F);
        assert_eq!((!a).u64(), 0xF0F0_FFFF_FFFF_0F0F);
        assert_eq!((!b).u64(), 0xFF00_FF00_FF00_FF00);
    }

    #[test]
    fn bitboard_shift_ops() {
        let a = Bitboard::new(0xFFFF_0000_0000_FFFF);
        let b = Bitboard::new(0x0000_0000_0000_0001);
        let c = Bitboard::new(0xF000_0000_0000_0000);

        assert_eq!((a << 1).u64(), 0xFFFE_0000_0001_FFFE);
        assert_eq!((a << 8).u64(), 0xFF00_0000_00FF_FF00);
        assert_eq!((b << 1).u64(), 0x2);
        assert_eq!((b << 8).u64(), 0x100);

        assert_eq!((a >> 1).u64(), 0x7FFF_8000_0000_7FFF);
        assert_eq!((a >> 8).u64(), 0x00FF_FF00_0000_00FF);
        assert_eq!((c >> 1).u64(), 0x7800_0000_0000_0000);
        assert_eq!((c >> 63).u64(), 0x1);
    }

    #[test]
    fn bitboard_assign_ops() {
        let mut b = Bitboard::new(0xFFFF_0000_0000_FFFF);
        b &= Bitboard::new(0xFF00_FF00_00FF_00FF);
        assert_eq!(b.u64(), 0xFF00_0000_0000_00FF);

        b |= Bitboard::new(0xFFF0_0F00_FF00_F000);
        assert_eq!(b.u64(), 0xFFF0_0F00_FF00_F0FF);

        b ^= Bitboard::new(0xFF00_F000_0FF0_00F0);
        assert_eq!(b.u64(), 0x00F0_FF00_F0F0_F00F);
    }

    // --- Bitboards ---
    #[rustfmt::skip]
    #[test]
    fn init_evasion_masks_correctly() {
        let bbs = Bitboards::init();
        
        for i in 0..64 {
            assert_eq!(bbs.evasion_masks[i][i].0, 0);
        }
    
        // Init same rank correctly
        assert_eq!(bbs.evasion_masks[Square::A1.idx()][Square::H1.idx()].0, 0xFE);
        assert_eq!(bbs.evasion_masks[Square::H6.idx()][Square::B6.idx()].0, 0x7E00_0000_0000);

        // Init same file correctly
        assert_eq!(bbs.evasion_masks[Square::B1.idx()][Square::B8.idx()].0, 0x0202_0202_0202_0200);
        assert_eq!(bbs.evasion_masks[Square::F7.idx()][Square::F2.idx()].0, 0x0000_2020_2020_2000);

        // Init same diag correctly
        assert_eq!(bbs.evasion_masks[Square::A1.idx()][Square::H8.idx()].0, 0x8040_2010_0804_0200);
        assert_eq!(bbs.evasion_masks[Square::E7.idx()][Square::B4.idx()].0, 0x0000_0804_0200_0000);

        // Non-inline squares return only the end square
        assert_eq!(bbs.evasion_masks[Square::B1.idx()][Square::C7.idx()].0, 0x0004_0000_0000_0000);
        assert_eq!(bbs.evasion_masks[Square::F7.idx()][Square::G2.idx()].0, 0x4000);
    }

    #[test]
    fn init_king_attacks_correctly() {
        let bbs = Bitboards::init();

        // Check that center squares are correct
        let expected_e5 = Square::D6.bitboard()
            | Square::E6.bitboard()
            | Square::F6.bitboard()
            | Square::F5.bitboard()
            | Square::F4.bitboard()
            | Square::E4.bitboard()
            | Square::D4.bitboard()
            | Square::D5.bitboard();

        assert_eq!(bbs.king_attacks[Square::E5.idx()], expected_e5);

        // Check that edge squares are correct
        let expected_a8 = Square::A7.bitboard() | Square::B7.bitboard() | Square::B8.bitboard();

        assert_eq!(bbs.king_attacks[Square::A8.idx()], expected_a8);
    }

    #[test]
    fn init_knight_attacks_correctly() {
        let bbs = Bitboards::init();

        // Check that center squares are correct
        let expected_d5 = Square::C7.bitboard()
            | Square::E7.bitboard()
            | Square::F6.bitboard()
            | Square::F4.bitboard()
            | Square::E3.bitboard()
            | Square::C3.bitboard()
            | Square::B4.bitboard()
            | Square::B6.bitboard();

        assert_eq!(bbs.knight_attacks[Square::D5.idx()], expected_d5);

        // Check that edge squares are correct
        let expected_h2 = Square::G4.bitboard() | Square::F3.bitboard() | Square::F1.bitboard();

        assert_eq!(bbs.knight_attacks[Square::H2.idx()], expected_h2);
    }

    #[test]
    fn init_pawn_attacks_correctly() {
        let bbs = Bitboards::init();

        // Ensure white and black attacks are not equal
        assert_ne!(
            bbs.pawn_attacks[Colour::White.idx()][Square::E4.idx()],
            bbs.pawn_attacks[Colour::Black.idx()][Square::E4.idx()]
        );

        // Check center attacks
        let white_expected_d4 = Square::C5.bitboard() | Square::E5.bitboard();
        let black_expected_d4 = Square::C3.bitboard() | Square::E3.bitboard();

        assert_eq!(
            bbs.pawn_attacks[Colour::White.idx()][Square::D4.idx()],
            white_expected_d4
        );
        assert_eq!(
            bbs.pawn_attacks[Colour::Black.idx()][Square::D4.idx()],
            black_expected_d4
        );

        // Check edge square attacks
        let white_expected_a5 = Square::B6.bitboard();
        let black_expected_h6 = Square::G5.bitboard();

        assert_eq!(
            bbs.pawn_attacks[Colour::White.idx()][Square::A5.idx()],
            white_expected_a5
        );
        assert_eq!(
            bbs.pawn_attacks[Colour::Black.idx()][Square::H6.idx()],
            black_expected_h6
        );

        // Check promotion rank squares
        let white_expected_c8 = Bitboard::new(0);
        let black_expected_d1 = Bitboard::new(0);

        assert_eq!(
            bbs.pawn_attacks[Colour::White.idx()][Square::C8.idx()],
            white_expected_c8
        );
        assert_eq!(
            bbs.pawn_attacks[Colour::Black.idx()][Square::D1.idx()],
            black_expected_d1
        );
    }
}
