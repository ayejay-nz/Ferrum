use crate::types::{Bitboard, Castling, Colour, NO_SQUARE, PieceCode, Square};

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
            pieces: [[0; 6]; 2],
            occupancy: [0; 3],
            mailbox: [PieceCode::EMPTY; 64],
            zkey: 0,
            fullmove_counter: 0,
            side_to_move: Colour::White,
            ep_square: NO_SQUARE,
            white_king_square: NO_SQUARE,
            black_king_square: NO_SQUARE,
            castling_rights: Castling::DEFAULT,
            halfmove_clock: 0,
        }
    }
}
