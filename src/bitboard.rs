use std::{
    arch::x86_64::_pext_u64,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr, Sub},
    sync::OnceLock,
};

use crate::types::{self, Colour, Direction, Piece, Square};

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const FILE_A: Bitboard = Bitboard(types::FILE_A);
    pub const FILE_B: Bitboard = Bitboard(types::FILE_B);
    pub const FILE_C: Bitboard = Bitboard(types::FILE_C);
    pub const FILE_D: Bitboard = Bitboard(types::FILE_D);
    pub const FILE_E: Bitboard = Bitboard(types::FILE_E);
    pub const FILE_F: Bitboard = Bitboard(types::FILE_F);
    pub const FILE_G: Bitboard = Bitboard(types::FILE_G);
    pub const FILE_H: Bitboard = Bitboard(types::FILE_H);

    pub const RANK_1: Bitboard = Bitboard(types::RANK_1);
    pub const RANK_2: Bitboard = Bitboard(types::RANK_2);
    pub const RANK_3: Bitboard = Bitboard(types::RANK_3);
    pub const RANK_4: Bitboard = Bitboard(types::RANK_4);
    pub const RANK_5: Bitboard = Bitboard(types::RANK_5);
    pub const RANK_6: Bitboard = Bitboard(types::RANK_6);
    pub const RANK_7: Bitboard = Bitboard(types::RANK_7);
    pub const RANK_8: Bitboard = Bitboard(types::RANK_8);

    pub const LIGHT_SQUARES: Bitboard = Bitboard(0x55AA_55AA_55AA_55AA);
    pub const DARK_SQUARES: Bitboard = Bitboard(0xAA55_AA55_AA55_AA55);

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
    pub fn msb(self) -> Square {
        debug_assert!(self.0 != 0);
        Square::new(63 - self.0.leading_zeros() as u8)
    }

    #[inline(always)]
    pub fn north_fill(self) -> Bitboard {
        let mut bb = self;
        bb |= bb << 8;
        bb |= bb << 16;
        bb |= bb << 32;
        bb
    }

    #[inline(always)]
    pub fn south_fill(self) -> Bitboard {
        let mut bb = self;
        bb |= bb >> 8;
        bb |= bb >> 16;
        bb |= bb >> 32;
        bb
    }

    #[inline(always)]
    pub fn frontfill(self, c: Colour) -> Bitboard {
        if c == Colour::White {
            self.north_fill()
        } else {
            self.south_fill()
        }
    }

    #[inline(always)]
    pub fn backfill(self, c: Colour) -> Bitboard {
        if c == Colour::White {
            self.south_fill()
        } else {
            self.north_fill()
        }
    }

    #[inline(always)]
    pub fn file(n: u8) -> Bitboard {
        debug_assert!(n < 8);

        match n {
            0 => Self::FILE_A,
            1 => Self::FILE_B,
            2 => Self::FILE_C,
            3 => Self::FILE_D,
            4 => Self::FILE_E,
            5 => Self::FILE_F,
            6 => Self::FILE_G,
            7 => Self::FILE_H,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn file_occupied(&self, file: u8) -> bool {
        debug_assert!(file < 8);

        let file_bb = Self::file(file);

        !(*self & file_bb).is_empty()
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

    #[inline(always)]
    pub fn bit_count(self) -> u32 {
        self.0.count_ones()
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

// bb - bb
impl Sub for Bitboard {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        // Use wrapping_sub because the Carry-Rippler trick relies on it
        Self(self.0.wrapping_sub(rhs.0))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Magic {
    offset: usize,
    mask: Bitboard,
}

impl Default for Magic {
    #[inline(always)]
    fn default() -> Self {
        Self {
            offset: 0,
            mask: Bitboard::new(0),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Bitboards {
    evasion_masks: [[Bitboard; 64]; 64],
    line_bbs: [[Bitboard; 64]; 64],
    knight_attacks: [Bitboard; 64],
    king_attacks: [Bitboard; 64],
    pawn_attacks: [[Bitboard; 64]; 2],
    bishop_masks: [Magic; 64],
    bishop_attacks: [Bitboard; 0x1480],
    rook_masks: [Magic; 64],
    rook_attacks: [Bitboard; 0x19000],
}

static BITBOARDS: OnceLock<Bitboards> = OnceLock::new();

pub fn bitboards() -> &'static Bitboards {
    BITBOARDS.get_or_init(Bitboards::init)
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

    fn sliding_attack(piece: Piece, sq: Square, occ: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::new(0);
        let rook_directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];
        let bishop_directions = [
            Direction::NorthEast,
            Direction::NorthWest,
            Direction::SouthEast,
            Direction::SouthWest,
        ];

        let directions = match piece {
            Piece::Rook => rook_directions,
            Piece::Bishop => bishop_directions,
            _ => unreachable!(),
        };

        for d in directions {
            let mut s = sq;
            while Self::safe_destination(s, d as i32) != Bitboard::new(0) {
                s = Square::new((s.u8() as i8 + d as i8) as u8);
                attacks |= s.bitboard();
                if occ & s.bitboard() != Bitboard::new(0) {
                    break;
                }
            }
        }

        attacks
    }

    fn init_pexts(piece: Piece, table: &mut [Bitboard], masks: &mut [Magic]) {
        let mut current_offset = 0;

        for sq in Square::ALL {
            // Board edges which are not considered for occupancy
            let edges = ((Bitboard::RANK_1 | Bitboard::RANK_8) & !sq.rank_bb())
                | ((Bitboard::FILE_A | Bitboard::FILE_H) & !sq.file_bb());

            let mask = Self::sliding_attack(piece, sq, Bitboard::new(0)) & !edges;
            masks[sq.idx()].mask = mask;
            masks[sq.idx()].offset = current_offset;

            // Use the Carry-Rippler trick to iterate over all sub-bitboards of mask
            let mut b = Bitboard::new(0);
            loop {
                unsafe {
                    table[current_offset + _pext_u64(b.u64(), mask.u64()) as usize] =
                        Self::sliding_attack(piece, sq, b)
                };

                b = (b - mask) & mask;

                if b.is_empty() {
                    break;
                }
            }

            current_offset += 1 << mask.bit_count();
        }
    }

    /// Initialise the various bitboard tables
    pub fn init() -> Self {
        let mut evasion_masks = [[Bitboard::new(0); 64]; 64];
        let mut line_bbs = [[Bitboard::new(0); 64]; 64];
        let mut knight_attacks = [Bitboard::new(0); 64];
        let mut king_attacks = [Bitboard::new(0); 64];
        let mut pawn_attacks = [[Bitboard::new(0); 64]; 2];

        let mut bishop_masks = [Magic::default(); 64];
        let mut bishop_attacks = [Bitboard::new(0); 0x1480];
        let mut rook_masks = [Magic::default(); 64];
        let mut rook_attacks = [Bitboard::new(0); 0x19000];

        // Generate PEXT bitboards for bishops and rooks
        Self::init_pexts(Piece::Bishop, &mut bishop_attacks, &mut bishop_masks);
        Self::init_pexts(Piece::Rook, &mut rook_attacks, &mut rook_masks);

        for s1 in Square::ALL {
            for s2 in Square::ALL {
                if s1 == s2 {
                    continue;
                }

                let same_rank = s1.rank() == s2.rank();
                let same_file = s1.file() == s2.file();
                let same_diag = s1.rank().abs_diff(s2.rank()) == s1.file().abs_diff(s2.file());

                if same_rank || same_file || same_diag {
                    // This can be tidied up with a global magics...
                    if same_rank || same_file {
                        let magic_s1 = rook_masks[s1.idx()];
                        let offset_s1 = magic_s1.offset;

                        let magic_s2 = rook_masks[s2.idx()];
                        let offset_s2 = magic_s2.offset;
                        line_bbs[s1.idx()][s2.idx()] = (rook_attacks[offset_s1]
                            & rook_attacks[offset_s2])
                            | s1.bitboard()
                            | s2.bitboard();
                    } else {
                        let magic_s1 = bishop_masks[s1.idx()];
                        let offset_s1 = magic_s1.offset;

                        let magic_s2 = bishop_masks[s2.idx()];
                        let offset_s2 = magic_s2.offset;
                        line_bbs[s1.idx()][s2.idx()] = (bishop_attacks[offset_s1]
                            & bishop_attacks[offset_s2])
                            | s1.bitboard()
                            | s2.bitboard();
                    }

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
            line_bbs,
            knight_attacks,
            king_attacks,
            pawn_attacks,
            bishop_masks,
            bishop_attacks,
            rook_masks,
            rook_attacks,
        }
    }

    /// Returns a bitboard representing the entire line (from the two edges of the board)
    /// which intersect the two squares. If the two squares are not on the same
    /// file/rank/diagonal it returns 0
    #[inline(always)]
    pub fn line_bb(&self, s1: Square, s2: Square) -> Bitboard {
        return self.line_bbs[s1.idx()][s2.idx()];
    }

    /// Returns a bitboard representing the squares semi-open segment between the two squares
    /// s1 and s2 (exc. s1 but inc. s2). If the two squares are not on the same file/rank/diagonal,
    /// it returns s2. This allows us to generate non-king evasion moves faster.
    #[inline(always)]
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

    #[inline(always)]
    pub fn bishop_attacks(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let magic = self.bishop_masks[sq.idx()];
        let mask = magic.mask;
        let offset = magic.offset;

        unsafe { self.bishop_attacks[offset + _pext_u64(occ.u64(), mask.u64()) as usize] }
    }

    #[inline(always)]
    pub fn rook_attacks(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let magic = self.rook_masks[sq.idx()];
        let mask = magic.mask;
        let offset = magic.offset;

        unsafe { self.rook_attacks[offset + _pext_u64(occ.u64(), mask.u64()) as usize] }
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
        let bbs = bitboards();
        
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

    #[rustfmt::skip]
    #[test]
    fn init_line_bbs_correct() {
        let bbs = bitboards();
        
        for i in 0..64 {
            assert_eq!(bbs.line_bbs[i][i].0, 0);
        }
    
        // Init same rank correctly
        assert_eq!(bbs.line_bbs[Square::A1.idx()][Square::H1.idx()].0, 0xFF);
        assert_eq!(bbs.line_bbs[Square::H6.idx()][Square::B6.idx()].0, 0xFF00_0000_0000);

        // Init same file correctly
        assert_eq!(bbs.line_bbs[Square::B1.idx()][Square::B8.idx()].0, 0x0202_0202_0202_0202);
        assert_eq!(bbs.line_bbs[Square::F7.idx()][Square::F2.idx()].0, 0x2020_2020_2020_2020);

        // Init same diag correctly
        assert_eq!(bbs.line_bbs[Square::B2.idx()][Square::G7.idx()].0, 0x8040_2010_0804_0201);
        assert_eq!(bbs.line_bbs[Square::E7.idx()][Square::B4.idx()].0, 0x2010_0804_0201_0000);

        // Non-inline squares return 0
        assert_eq!(bbs.line_bbs[Square::B1.idx()][Square::C7.idx()].0, 0);
        assert_eq!(bbs.line_bbs[Square::F7.idx()][Square::G2.idx()].0, 0);
    }

    #[test]
    fn init_king_attacks_correctly() {
        let bbs = bitboards();

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
        let bbs = bitboards();

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
        let bbs = bitboards();

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
