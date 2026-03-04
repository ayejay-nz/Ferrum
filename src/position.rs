use crate::types::{Bitboard, Castling, Colour, Piece, PieceCode, Square};

pub struct Position {
    pub pieces: [[Bitboard; 6]; 2],
    pub occupancy: [Bitboard; 3],
    pub mailbox: [PieceCode; 64],
    pub zkey: u64,
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
            mailbox: [PieceCode::EMPTY; 64],
            zkey: 0,
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
        self.mailbox[square.idx()] = PieceCode::new(colour, piece);

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
        self.mailbox[square.idx()] = PieceCode::EMPTY;
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

        return position;
    }
}
