use crate::types::{Colour, Square};

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

#[cfg(test)]
mod test {
    use super::*;

    use crate::position::{Position, StateInfo};

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

        // Should panic
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
}
