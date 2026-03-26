use std::cmp::Reverse;

use crate::{
    movegen::{MoveList, generate_evasions, generate_noisy, generate_quiets},
    position::Position,
    search::OrderingTables,
    types::Move,
};

#[derive(Eq, PartialEq, Debug)]
enum Stage {
    // Generate regular search moves
    PVMove,
    TTMove,
    GenNoisy,
    Noisy,
    GenQuiets,
    Quiets,

    // Generate evasion moves
    PVEvasion,
    TTEvasion,
    GenEvasions,
    Evasions,

    // Generate q search moves
    TTQSearch,
    GenQSearch,
    QSearch,

    Done,
}

impl Stage {
    #[inline(always)]
    fn next(&mut self) {
        *self = match *self {
            Self::PVMove => Self::TTMove,
            Self::TTMove => Self::GenNoisy,
            Self::GenNoisy => Self::Noisy,
            Self::Noisy => Self::GenQuiets,
            Self::GenQuiets => Self::Quiets,
            Self::Quiets => Self::Done,

            Self::PVEvasion => Self::TTEvasion,
            Self::TTEvasion => Self::GenEvasions,
            Self::GenEvasions => Self::Evasions,
            Self::Evasions => Self::Done,

            Self::TTQSearch => Self::GenQSearch,
            Self::GenQSearch => Self::QSearch,
            Self::QSearch => Self::Done,

            Self::Done => Self::Done,
        }
    }
}

pub struct MovePicker {
    stage: Stage,
    pv_move: Move,
    tt_move: Move,
    moves: MoveList,
    ply: usize,
    idx: usize,
}

impl MovePicker {
    fn start_stage(in_check: bool, pv_move: Move, tt_move: Move, depth: i32) -> Stage {
        if in_check {
            if !pv_move.is_null() {
                Stage::PVEvasion
            } else if !tt_move.is_null() {
                Stage::TTEvasion
            } else {
                Stage::GenEvasions
            }
        } else if depth > 0 {
            if !pv_move.is_null() {
                Stage::PVMove
            } else if !tt_move.is_null() {
                Stage::TTMove
            } else {
                Stage::GenNoisy
            }
        } else if tt_move.is_null() {
            Stage::GenQSearch
        } else {
            Stage::TTQSearch
        }
    }

    pub fn new(in_check: bool, pv_move: Move, tt_move: Move, depth: i32, ply: usize) -> Self {
        let start_stage = Self::start_stage(in_check, pv_move, tt_move, depth);

        Self {
            stage: start_stage,
            pv_move,
            tt_move,
            moves: MoveList::new(),
            ply,
            idx: 0,
        }
    }

    pub fn next(&mut self, pos: &Position, ordering: &OrderingTables) -> Option<Move> {
        match self.stage {
            // Get PV move
            Stage::PVMove | Stage::PVEvasion => {
                self.stage.next();
                self.idx = 0;

                if self.pv_move.is_null() {
                    return self.next(pos, ordering);
                }

                Some(self.pv_move)
            }

            // Get TT move
            Stage::TTMove | Stage::TTEvasion | Stage::TTQSearch => {
                self.stage.next();
                self.idx = 0;

                if self.tt_move.is_null() || self.tt_move == self.pv_move {
                    return self.next(pos, ordering);
                }

                Some(self.tt_move)
            }

            // Generate regular search moves
            Stage::GenNoisy => {
                self.stage.next();
                generate_noisy(pos, &mut self.moves);

                self.moves
                    .as_mut_slice()
                    .sort_unstable_by_key(|&mv| Reverse(ordering.score_noisy(pos, mv)));
                self.next(pos, ordering)
            }
            Stage::Noisy => {
                while self.idx < self.moves.len() {
                    let mv = self.moves.as_slice()[self.idx];
                    self.idx += 1;

                    if mv == self.tt_move || mv == self.pv_move {
                        continue;
                    }

                    return Some(mv);
                }

                self.stage.next();
                self.idx = 0;
                return self.next(pos, ordering);
            }
            Stage::GenQuiets => {
                self.stage.next();
                generate_quiets(pos, &mut self.moves);

                self.moves
                    .as_mut_slice()
                    .sort_unstable_by_key(|&mv| Reverse(ordering.score_quiet(pos, mv, self.ply)));
                self.next(pos, ordering)
            }
            Stage::Quiets => {
                while self.idx < self.moves.len() {
                    let mv = self.moves.as_slice()[self.idx];
                    self.idx += 1;

                    if mv == self.tt_move || mv == self.pv_move {
                        continue;
                    }

                    return Some(mv);
                }

                self.stage.next();
                self.idx = 0;
                return self.next(pos, ordering);
            }

            // Generate evasion moves
            Stage::GenEvasions => {
                self.stage.next();
                generate_evasions(pos, &mut self.moves);

                self.moves
                    .as_mut_slice()
                    .sort_unstable_by_key(|&mv| Reverse(ordering.score_evasion(pos, mv, self.ply)));
                self.next(pos, ordering)
            }
            Stage::Evasions => {
                while self.idx < self.moves.len() {
                    let mv = self.moves.as_slice()[self.idx];
                    self.idx += 1;

                    if mv == self.tt_move || mv == self.pv_move {
                        continue;
                    }

                    return Some(mv);
                }

                self.stage.next();
                self.idx = 0;
                return self.next(pos, ordering);
            }

            // Generate q search moves
            Stage::GenQSearch => {
                self.stage.next();
                generate_noisy(pos, &mut self.moves);

                self.moves
                    .as_mut_slice()
                    .sort_unstable_by_key(|&mv| Reverse(ordering.score_noisy(pos, mv)));
                self.next(pos, ordering)
            }
            Stage::QSearch => {
                while self.idx < self.moves.len() {
                    let mv = self.moves.as_slice()[self.idx];
                    self.idx += 1;

                    if mv == self.tt_move {
                        continue;
                    }

                    return Some(mv);
                }

                self.stage.next();
                self.idx = 0;
                return self.next(pos, ordering);
            }

            Stage::Done => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{MoveFlag, Square};

    use super::*;

    #[test]
    fn stage_next_is_correct() {
        let mut stage = Stage::TTMove;
        stage.next();
        assert_eq!(stage, Stage::GenNoisy);
        stage.next();
        assert_eq!(stage, Stage::Noisy);
        stage.next();
        assert_eq!(stage, Stage::GenQuiets);
        stage.next();
        assert_eq!(stage, Stage::Quiets);
        stage.next();
        assert_eq!(stage, Stage::Done);

        stage = Stage::PVEvasion;
        stage.next();
        assert_eq!(stage, Stage::TTEvasion);
        stage.next();
        assert_eq!(stage, Stage::GenEvasions);
        stage.next();
        assert_eq!(stage, Stage::Evasions);
        stage.next();
        assert_eq!(stage, Stage::Done);

        stage = Stage::TTQSearch;
        stage.next();
        assert_eq!(stage, Stage::GenQSearch);
        stage.next();
        assert_eq!(stage, Stage::QSearch);
        stage.next();
        assert_eq!(stage, Stage::Done);
    }

    #[test]
    fn move_picker_next_is_correct() {
        let pos = Position::from_fen("7k/1P4b1/8/8/4N3/2p5/8/K7 w - - 0 1");
        let in_check = !pos.checkers.is_empty();
        let depth = 3;
        let ply = 2;

        let mut ordering = OrderingTables::new();
        let k1 = Move::new(Square::E4, Square::F6, MoveFlag::Quiet);
        ordering.update_killers(k1, ply);

        let tt_move = Move::new(Square::B7, Square::B8, MoveFlag::PromoQ);
        let mut mp = MovePicker::new(in_check, Move::NULL, tt_move, depth, ply);

        // Correctly skips pv move and gets tt move and updates state
        assert_eq!(mp.next(&pos, &ordering).unwrap(), tt_move);

        assert_eq!(mp.stage, Stage::GenNoisy);
        assert_eq!(mp.idx, 0);

        // Correctly gets noisy moves
        let n1 = Move::new(Square::E4, Square::C3, MoveFlag::Capture);
        let n2 = Move::new(Square::B7, Square::B8, MoveFlag::PromoN);
        let n3 = Move::new(Square::B7, Square::B8, MoveFlag::PromoR);
        let n4 = Move::new(Square::B7, Square::B8, MoveFlag::PromoB);

        assert_eq!(mp.next(&pos, &ordering).unwrap(), n1);
        assert_eq!(mp.next(&pos, &ordering).unwrap(), n2);
        assert_eq!(mp.next(&pos, &ordering).unwrap(), n3);
        assert_eq!(mp.next(&pos, &ordering).unwrap(), n4);

        assert_eq!(mp.stage, Stage::Noisy);
        assert_eq!(mp.idx, 5);

        // Correctly gets killers
        assert_eq!(mp.next(&pos, &ordering).unwrap(), k1);

        assert_eq!(mp.stage, Stage::Quiets);
        assert_eq!(mp.idx, 1);

        // Correct gets remaining quiets
        let quiet_count = 8;
        for i in 1..=quiet_count {
            assert!(mp.next(&pos, &ordering).is_some());

            assert_eq!(mp.idx, i + 1);
            assert_eq!(mp.stage, Stage::Quiets);
        }

        assert!(mp.next(&pos, &ordering).is_none());
        assert_eq!(mp.stage, Stage::Done);
        assert_eq!(mp.idx, 0);
    }
}
