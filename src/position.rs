use crate::{
    types::{Bitboard, Castling, Colour, Mailbox, Move, Piece, PieceCode, Square},
    zobrist::{ZKey, ep_hashable},
};

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StateInfo {
    pub zkey: ZKey,
    pub ep_square: Square,
    pub castling_rights: Castling,
    pub captured_piece: Option<Piece>,
    pub halfmove_clock: u8,
    pub fullmove_counter: u16,
}

impl StateInfo {
    pub fn new() -> Self {
        Self {
            zkey: ZKey(0),
            ep_square: Square::NONE,
            castling_rights: Castling::NONE,
            captured_piece: None,
            halfmove_clock: 0,
            fullmove_counter: 0,
        }
    }

    pub fn set_from_position(&mut self, pos: &Position) {
        self.zkey = pos.zkey;
        self.ep_square = pos.ep_square;
        self.castling_rights = pos.castling_rights;
        self.captured_piece = None;
        self.halfmove_clock = pos.halfmove_clock;
        self.fullmove_counter = pos.fullmove_counter;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Position {
    pub pieces: [[Bitboard; 6]; 2],
    pub occupancy: [Bitboard; 3],
    pub mailbox: Mailbox,
    pub zkey: ZKey,
    pub fullmove_counter: u16,
    pub side_to_move: Colour,
    pub ep_square: Square,
    pub white_king_square: Square,
    pub black_king_square: Square,
    pub castling_rights: Castling,
    pub halfmove_clock: u8,
}

impl Position {
    pub fn new() -> Self {
        let mut zkey = ZKey(0);
        zkey.toggle_side();

        Self {
            pieces: [[Bitboard::new(0); 6]; 2],
            occupancy: [Bitboard::new(0); 3],
            mailbox: Mailbox::new(),
            zkey: zkey,
            fullmove_counter: 0,
            side_to_move: Colour::White,
            ep_square: Square::NONE,
            white_king_square: Square::NONE,
            black_king_square: Square::NONE,
            castling_rights: Castling::NONE,
            halfmove_clock: 0,
        }
    }

    #[inline(always)]
    pub fn place_piece(&mut self, colour: Colour, piece: Piece, square: Square) {
        self.pieces[colour.idx()][piece.idx()].set_square(square);
        self.occupancy[colour.idx()].set_square(square);
        self.occupancy[2].set_square(square);

        let pc = PieceCode::new(colour, piece);
        self.mailbox.set_square(square, pc);

        self.zkey.toggle_piece(pc, square);

        // Set white/black king squares
        if piece == Piece::King {
            match colour {
                Colour::White => self.white_king_square = square,
                Colour::Black => self.black_king_square = square,
            }
        }
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, colour: Colour, piece: Piece, square: Square) {
        self.pieces[colour.idx()][piece.idx()].clear_square(square);
        self.occupancy[colour.idx()].clear_square(square);
        self.occupancy[2].clear_square(square);
        self.mailbox.clear_square(square);

        let pc = PieceCode::new(colour, piece);
        self.zkey.toggle_piece(pc, square);
    }

    #[inline(always)]
    pub fn make_move(&mut self, mv: Move, state: &mut StateInfo) {
        state.zkey = self.zkey;
        state.ep_square = self.ep_square;
        state.castling_rights = self.castling_rights;
        state.halfmove_clock = self.halfmove_clock;
        state.fullmove_counter = self.fullmove_counter;
        state.captured_piece = None;

        // Unhash old en passant file if it exists
        if ep_hashable(&self.mailbox, self.ep_square, self.side_to_move) {
            self.zkey.toggle_ep_file(self.ep_square);
        }
        self.ep_square = Square::NONE;

        self.halfmove_clock += 1;
        if self.side_to_move == Colour::Black {
            self.fullmove_counter += 1;
        }

        let colour = self.side_to_move;
        let from = mv.from();
        let to = mv.to();
        let piece = self.mailbox.piece_at(from).unwrap();

        // Update castling rights and increment zobrist key
        self.zkey.toggle_castling(self.castling_rights);
        self.castling_rights.update(from, to);
        self.zkey.toggle_castling(self.castling_rights);

        // If promotion occurs we need to set the promoted piece bitboard and mailbox code
        let to_piece = mv.promotion_piece().unwrap_or(piece);

        // Remove captured piece and update halfmove clock
        if mv.is_capture() {
            self.halfmove_clock = 0;

            // Find the captured piece -- ep piece is not always on `to` square
            let capture_square = if mv.is_ep_capture() {
                mv.get_ep_pawn_square()
            } else {
                to
            };
            let captured_piece = self.mailbox.piece_at(capture_square).unwrap();

            self.remove_piece(colour.opposite(), captured_piece, capture_square);
            state.captured_piece = Some(captured_piece);
        }

        // Move the piece
        self.remove_piece(colour, piece, from);
        self.place_piece(colour, to_piece, to);

        // Move rook if it was a castling move
        if let Some(side) = mv.castle_type() {
            let (rook_from, rook_to) = Castling::get_rook_squares_from_castle(colour, side);
            self.remove_piece(colour, Piece::Rook, rook_from);
            self.place_piece(colour, Piece::Rook, rook_to);
        }

        // Pawn move resets halfmove clock
        if piece == Piece::Pawn {
            self.halfmove_clock = 0;

            // If move was a double pawn move, update ep square
            if mv.is_double_push() {
                let ep_square = Square::new((from.u8() + to.u8()) >> 1);

                // Update the zkey only if the double pushed pawn has an opposition pawn next to it
                if ep_hashable(&self.mailbox, ep_square, colour.opposite()) {
                    self.zkey.toggle_ep_file(ep_square);
                }

                self.ep_square = ep_square;
            }
        }

        self.side_to_move = self.side_to_move.opposite();
        self.zkey.toggle_side();
    }

    #[inline(always)]
    pub fn undo_move(&mut self, mv: Move, prev: &StateInfo) {
        self.side_to_move = self.side_to_move.opposite();

        self.halfmove_clock = prev.halfmove_clock;
        self.fullmove_counter = prev.fullmove_counter;
        self.castling_rights = prev.castling_rights;
        self.ep_square = prev.ep_square;

        let colour = self.side_to_move;
        let from = mv.from();
        let to = mv.to();
        let mut piece = self.mailbox.piece_at(to).unwrap();

        // Move piece back to its original square
        self.remove_piece(colour, piece, to);
        // If move was a promotion, we want to place back a pawn
        if mv.is_promotion() {
            piece = Piece::Pawn
        }
        self.place_piece(colour, piece, from);

        // Place back captured piece
        if mv.is_capture() {
            let capture_square = if mv.is_ep_capture() {
                mv.get_ep_pawn_square()
            } else {
                to
            };

            self.place_piece(
                colour.opposite(),
                prev.captured_piece.unwrap(),
                capture_square,
            );
        }

        // If move was a castle, place back the rook
        if mv.is_castle() {
            let (rook_from, rook_to) =
                Castling::get_rook_squares_from_castle(colour, mv.castle_type().unwrap());

            self.remove_piece(colour, Piece::Rook, rook_to);
            self.place_piece(colour, Piece::Rook, rook_from);
        }

        // Replace zkey at the end so we don't accidentally update it placing/removing pieces
        self.zkey = prev.zkey;
    }

    pub fn from_fen(fen: &str) -> Self {
        // 0 - piece placement
        // 1 - side to move
        // 2 - castling
        // 3 - en passant
        // 4 - halfmove count
        // 5 - fullmove count
        let fen_parts: Vec<&str> = fen.split_ascii_whitespace().collect();
        let mut position = Position::new();

        // Piece placement
        let mut rank = 7u8;
        let mut file = 0u8;
        for c in fen_parts[0].chars() {
            if c == '/' {
                rank -= 1;
                file = 0;
            } else if let Some(digit) = c.to_digit(10) {
                file += digit as u8;
            } else if let Some(pc) = PieceCode::from_char(c) {
                // Since pc was just built from valid enums, these are safe
                let piece = pc.piece().unwrap();
                let colour = pc.colour().unwrap();
                let square = Square::from_coords(rank, file);

                position.place_piece(colour, piece, square);
                file += 1;
            }
        }

        // Side to move
        position.side_to_move = match fen_parts[1] {
            "w" => Colour::White,
            "b" => Colour::Black,
            _ => unreachable!(),
        };

        // Castling
        let mut rights = 0;
        if fen_parts[2] != "-" {
            for c in fen_parts[2].chars() {
                match c {
                    'K' => rights |= Castling::WK_BIT,
                    'Q' => rights |= Castling::WQ_BIT,
                    'k' => rights |= Castling::BK_BIT,
                    'q' => rights |= Castling::BQ_BIT,
                    _ => unreachable!(),
                }
            }
        }
        position.castling_rights = Castling::new(rights);

        // En passant
        position.ep_square = if fen_parts[3] == "-" {
            Square::NONE
        } else {
            let file = (fen_parts[3].as_bytes()[0]) - b'a';
            let rank = (fen_parts[3].as_bytes()[1]) - b'1';
            Square::from_coords(rank, file)
        };

        // Half/fullmove count
        position.halfmove_clock = fen_parts[4].parse::<u8>().unwrap();
        position.fullmove_counter = fen_parts[5].parse::<u16>().unwrap();

        // Zobrist key
        position.zkey = ZKey::compute_zobrist_key(
            &position.mailbox,
            position.side_to_move,
            position.castling_rights,
            position.ep_square,
        );

        return position;
    }

    pub fn default() -> Self {
        Self::from_fen(DEFAULT_FEN)
    }

    pub fn display(&self) {
        let piece_symbols = [
            ['♟', '♞', '♝', '♜', '♛', '♚'], // These are technically black pieces according to ASCII
            ['♙', '♘', '♗', '♖', '♕', '♔'], // But I'm not falling for that propaganda
        ];

        println!("\n  +-----------------+");

        // We want to print white on the bottom, so start with 8th rank
        for rank in (0..8).rev() {
            print!("{} | ", rank + 1);

            for file in 0..8 {
                let square = Square::from_coords(rank, file);
                let piece = self.mailbox.piece_code_at(square);

                if piece.is_empty() {
                    print!(". ");
                } else {
                    let symbol =
                        piece_symbols[piece.colour().unwrap().idx()][piece.piece().unwrap().idx()];
                    print!("{} ", symbol);
                }
            }
            println!("|");
        }

        println!("  +-----------------+");
        println!("    a b c d e f g h");
    }
}

#[cfg(test)]
mod test {
    use crate::types::MoveFlag;

    use super::*;

    struct GoalState {
        ep_square: Square,
        castling_rights: u8,
        halfmove_clock: u8,
        fullmove_counter: u16,
    }

    fn check_state_correctness(pos: &Position, state: &StateInfo, goal: &GoalState) {
        assert_eq!(
            state.zkey,
            ZKey::compute_zobrist_key(
                &pos.mailbox,
                pos.side_to_move,
                pos.castling_rights,
                pos.ep_square
            )
        );
        assert_eq!(state.ep_square, goal.ep_square);
        assert_eq!(state.castling_rights.bits(), goal.castling_rights);
        assert_eq!(state.captured_piece, None);
        assert_eq!(state.halfmove_clock, goal.halfmove_clock);
        assert_eq!(state.fullmove_counter, goal.fullmove_counter);
    }

    fn check_position_correctness(actual: &Position, expected: &Position) {
        assert_eq!(actual.pieces, expected.pieces);
        assert_eq!(actual.occupancy, expected.occupancy);
        assert_eq!(actual.mailbox, expected.mailbox);

        assert_eq!(actual.white_king_square, expected.white_king_square);
        assert_eq!(actual.black_king_square, expected.black_king_square);
        assert_eq!(actual.zkey, expected.zkey);

        // Other fields which are not touched by place_piece
        assert_eq!(actual.side_to_move, expected.side_to_move);
        assert_eq!(actual.ep_square, expected.ep_square);
        assert_eq!(actual.halfmove_clock, expected.halfmove_clock);
        assert_eq!(actual.fullmove_counter, expected.fullmove_counter);
        assert_eq!(actual.castling_rights, expected.castling_rights);
    }

    struct MoveCase {
        name: &'static str,
        start_fen: &'static str,
        mv: Move,
        expected_fen: &'static str,
        expected_captured_piece: Option<Piece>,
    }

    const MOVE_CASES: &[MoveCase] = &[
        MoveCase {
            name: "quiet_knight_move",
            start_fen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            mv: Move::new(Square::B8, Square::C6, MoveFlag::Quiet),
            expected_fen: "r1bqkbnr/pppppppp/2n5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2",
            expected_captured_piece: None,
        },
        MoveCase {
            name: "double_push_set_ep",
            start_fen: DEFAULT_FEN,
            mv: Move::new(Square::D2, Square::D4, MoveFlag::DoublePush),
            expected_fen: "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1",
            expected_captured_piece: None,
        },
        MoveCase {
            name: "ep_capture",
            start_fen: "rnbqkbnr/1ppppppp/8/pP6/8/8/P1PPPPPP/RNBQKBNR w KQkq a6 0 4",
            mv: Move::new(Square::B5, Square::A6, MoveFlag::EpCapture),
            expected_fen: "rnbqkbnr/1ppppppp/P7/8/8/8/P1PPPPPP/RNBQKBNR b KQkq - 0 4",
            expected_captured_piece: Some(Piece::Pawn),
        },
        MoveCase {
            name: "normal_capture",
            start_fen: "rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b KQkq c3 0 2",
            mv: Move::new(Square::D5, Square::C4, MoveFlag::Capture),
            expected_fen: "rnbqkbnr/ppp1pppp/8/8/2pP4/8/PP2PPPP/RNBQKBNR w KQkq - 0 3",
            expected_captured_piece: Some(Piece::Pawn),
        },
        MoveCase {
            name: "promotion_to_queen",
            start_fen: "r1bqkbnr/pPpppppp/3n4/8/8/8/PP1PPPPP/RNBQKBNR w KQkq - 1 5",
            mv: Move::new(Square::B7, Square::B8, MoveFlag::PromoQ),
            expected_fen: "rQbqkbnr/p1pppppp/3n4/8/8/8/PP1PPPPP/RNBQKBNR b KQkq - 0 5",
            expected_captured_piece: None,
        },
        MoveCase {
            name: "capturing_promotion_to_knight",
            start_fen: "r1bqkbnr/pPpppppp/3n4/8/8/8/PP1PPPPP/RNBQKBNR w KQkq - 1 5",
            mv: Move::new(Square::B7, Square::A8, MoveFlag::PromoCaptureN),
            expected_fen: "N1bqkbnr/p1pppppp/3n4/8/8/8/PP1PPPPP/RNBQKBNR b KQk - 0 5",
            expected_captured_piece: Some(Piece::Rook),
        },
        MoveCase {
            name: "black_kingside_castle",
            start_fen: "rnbqk2r/ppp1bppp/4pn2/3p2B1/2PP4/2N1P3/PP3PPP/R2QKBNR b KQkq - 0 5",
            mv: Move::new(Square::E8, Square::G8, MoveFlag::KingCastle),
            expected_fen: "rnbq1rk1/ppp1bppp/4pn2/3p2B1/2PP4/2N1P3/PP3PPP/R2QKBNR w KQ - 1 6",
            expected_captured_piece: None,
        },
        MoveCase {
            name: "white_queenside_castle",
            start_fen: "rnbqkb1r/1pp2ppp/p3pn2/3p4/3P1B2/2N5/PPPQPPPP/R3KBNR w KQkq - 0 5",
            mv: Move::new(Square::E1, Square::C1, MoveFlag::QueenCastle),
            expected_fen: "rnbqkb1r/1pp2ppp/p3pn2/3p4/3P1B2/2N5/PPPQPPPP/2KR1BNR b kq - 1 5",
            expected_captured_piece: None,
        },
    ];

    #[test]
    fn stateinfo_set_from_position_is_correct() {
        // Check against default position
        let pos = Position::default();
        let mut state = StateInfo::new();
        state.set_from_position(&pos);

        let goal = GoalState {
            ep_square: Square::NONE,
            castling_rights: Castling::DEFAULT.bits(),
            halfmove_clock: 0,
            fullmove_counter: 1,
        };
        check_state_correctness(&pos, &state, &goal);

        // Update state to new position
        let fen = "rnbq1rk1/ppp2ppp/4pn2/b2p2B1/1PPP4/P1N2N2/4PPPP/R2QKB1R b KQ b3 0 7";
        let pos = Position::from_fen(&fen);
        state.set_from_position(&pos);

        let goal = GoalState {
            ep_square: Square::B3,
            castling_rights: Castling::WK_BIT | Castling::WQ_BIT,
            halfmove_clock: 0,
            fullmove_counter: 7,
        };
        check_state_correctness(&pos, &state, &goal);
    }

    #[test]
    fn position_place_piece_is_correct() {
        let mut pos = Position::new();

        pos.place_piece(Colour::White, Piece::King, Square::E1);
        pos.place_piece(Colour::Black, Piece::King, Square::E8);
        pos.place_piece(Colour::White, Piece::Rook, Square::A1);
        pos.place_piece(Colour::Black, Piece::Rook, Square::H8);
        pos.place_piece(Colour::White, Piece::Bishop, Square::G5);
        pos.place_piece(Colour::Black, Piece::Bishop, Square::G7);
        pos.place_piece(Colour::White, Piece::Knight, Square::H3);
        pos.place_piece(Colour::Black, Piece::Knight, Square::C6);
        pos.place_piece(Colour::White, Piece::Pawn, Square::E2);
        pos.place_piece(Colour::Black, Piece::Pawn, Square::E7);

        let expected_fen = "4k2r/4p1b1/2n5/6B1/8/7N/4P3/R3K3 w - - 0 0";
        let expected_pos = Position::from_fen(expected_fen);
        check_position_correctness(&pos, &expected_pos);
    }

    #[test]
    fn position_remove_piece_is_correct() {
        let mut pos = Position::default();

        pos.remove_piece(Colour::White, Piece::Pawn, Square::H2);
        pos.remove_piece(Colour::White, Piece::Pawn, Square::D2);
        pos.remove_piece(Colour::White, Piece::Rook, Square::A1);
        pos.remove_piece(Colour::White, Piece::Queen, Square::D1);
        pos.remove_piece(Colour::Black, Piece::Pawn, Square::D7);
        pos.remove_piece(Colour::Black, Piece::Pawn, Square::G7);
        pos.remove_piece(Colour::Black, Piece::Knight, Square::B8);
        pos.remove_piece(Colour::Black, Piece::Bishop, Square::F8);

        // Note that remove_piece does not update castling rights
        let expected_fen = "r1bqk1nr/ppp1pp1p/8/8/8/8/PPP1PPP1/1NB1KBNR w KkQq - 0 1";
        let expected_pos = Position::from_fen(expected_fen);
        check_position_correctness(&pos, &expected_pos);
    }

    #[test]
    fn position_make_move_is_correct() {
        for case in MOVE_CASES {
            let mut pos = Position::from_fen(case.start_fen);
            let start = Position::from_fen(case.start_fen);
            let mut state = StateInfo::new();
            state.set_from_position(&pos);

            pos.make_move(case.mv, &mut state);
            let expected_pos = Position::from_fen(case.expected_fen);

            check_position_correctness(&pos, &expected_pos);

            // Validate state snapshot
            assert_eq!(state.zkey, start.zkey, "failed case: {}", case.name);
            assert_eq!(
                state.ep_square, start.ep_square,
                "failed case: {}",
                case.name
            );
            assert_eq!(
                state.castling_rights, start.castling_rights,
                "failed case: {}",
                case.name
            );
            assert_eq!(
                state.halfmove_clock, start.halfmove_clock,
                "failed case: {}",
                case.name
            );
            assert_eq!(
                state.fullmove_counter, start.fullmove_counter,
                "failed case: {}",
                case.name
            );
            assert_eq!(
                state.captured_piece, case.expected_captured_piece,
                "failed case: {}",
                case.name
            );
        }
    }

    #[test]
    fn position_undo_move_is_correct() {
        for case in MOVE_CASES {
            let mut pos = Position::from_fen(case.start_fen);
            let start = Position::from_fen(case.start_fen);
            let mut state = StateInfo::new();
            state.set_from_position(&pos);

            pos.make_move(case.mv, &mut state);
            pos.undo_move(case.mv, &state);

            check_position_correctness(&pos, &start);
        }
    }
}
