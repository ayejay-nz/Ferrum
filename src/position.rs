use crate::{
    types::{Bitboard, Castling, Colour, Mailbox, Move, Piece, PieceCode, Square},
    zobrist::{ZKey, ep_hashable},
};

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub struct StateInfo {
    pub zkey: ZKey,
    pub ep_square: Square,
    pub castling_rights: Castling,
    pub captured_piece: Option<Piece>,
    pub halfmove_clock: u8,
    pub fullmove_counter: u16,
}

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
        Self {
            pieces: [[Bitboard::new(0); 6]; 2],
            occupancy: [Bitboard::new(0); 3],
            mailbox: Mailbox::new(),
            zkey: ZKey(0),
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

    pub fn load_fen(fen: &str) -> Self {
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
