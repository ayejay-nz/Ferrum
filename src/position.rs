use crate::types::{Bitboard, Castling, Colour, Piece, Square};

pub struct Position {
    pub pieces: [[Bitboard; 6]; 2],
    pub occupancy: [Bitboard; 3],
    pub mailbox: [Piece; 64],
    pub zkey: u64,
    pub fullmove_counter: u16,
    pub side_to_move: Colour,
    pub ep_square: Square,
    pub white_king_square: Square,
    pub black_king_square: Square,
    pub castling_rights: Castling,
    pub halfmove_clock: u8,
}
