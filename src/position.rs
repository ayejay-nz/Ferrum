use crate::{
    bitboard::{Bitboard, bitboards},
    types::{Castling, CastlingType, Colour, Direction, Mailbox, Move, Piece, PieceCode, Square},
    zobrist::{ZKey, ep_hashable},
};

pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const WHITE_OO_PATH: Bitboard = Bitboard::new(0x60);
const WHITE_OOO_PATH: Bitboard = Bitboard::new(0xE);
const BLACK_OO_PATH: Bitboard = Bitboard::new(0x60 << 56);
const BLACK_OOO_PATH: Bitboard = Bitboard::new(0xE << 56);

const fn build_castling_paths() -> [Bitboard; 16] {
    let mut paths = [Bitboard::new(0); 16];
    let mut i = 0;

    while i < 16 {
        let rights = Castling::new(i as u8);
        let mut path = 0u64;

        if rights.can_white_ks() {
            path |= WHITE_OO_PATH.u64()
        }
        if rights.can_white_qs() {
            path |= WHITE_OOO_PATH.u64()
        }
        if rights.can_black_ks() {
            path |= BLACK_OO_PATH.u64()
        }
        if rights.can_black_qs() {
            path |= BLACK_OOO_PATH.u64()
        }

        paths[i] = Bitboard::new(path);
        i += 1;
    }

    paths
}

const CASTLING_PATHS: [Bitboard; 16] = build_castling_paths();

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
    pub pinned_pieces: [Bitboard; 2],
    pub pinners: [Bitboard; 2],
    pub checkers: Bitboard,
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
            pinned_pieces: [Bitboard::new(0); 2],
            pinners: [Bitboard::new(0); 2],
            checkers: Bitboard::new(0),
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
    pub fn king_square(&self, c: Colour) -> Square {
        if c == Colour::White {
            return self.white_king_square;
        } else {
            return self.black_king_square;
        }
    }

    #[inline(always)]
    pub fn bishop_sliders(&self, c: Colour) -> Bitboard {
        self.pieces[c.idx()][Piece::Bishop.idx()] | self.pieces[c.idx()][Piece::Queen.idx()]
    }

    #[inline(always)]
    pub fn rook_sliders(&self, c: Colour) -> Bitboard {
        self.pieces[c.idx()][Piece::Rook.idx()] | self.pieces[c.idx()][Piece::Queen.idx()]
    }

    #[inline(always)]
    pub fn place_piece(&mut self, colour: Colour, piece: Piece, square: Square) {
        self.pieces[colour.idx()][piece.idx()].set_square(square);
        self.occupancy[colour.idx()].set_square(square);
        self.occupancy[2].set_square(square);

        let pc = PieceCode::new(colour, piece);
        self.mailbox.set_square(square, pc);

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

        let us = self.side_to_move;
        let them = us.opposite();
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

            self.remove_piece(them, captured_piece, capture_square);
            self.zkey
                .toggle_piece(PieceCode::new(them, captured_piece), capture_square);
            state.captured_piece = Some(captured_piece);
        }

        // Move the piece
        self.remove_piece(us, piece, from);
        self.zkey.toggle_piece(PieceCode::new(us, piece), from);
        self.place_piece(us, to_piece, to);
        self.zkey.toggle_piece(PieceCode::new(us, to_piece), to);

        // Move rook if it was a castling move
        if let Some(side) = mv.castle_type() {
            let (rook_from, rook_to) = Castling::get_rook_squares_from_castle(us, side);
            self.remove_piece(us, Piece::Rook, rook_from);
            self.zkey
                .toggle_piece(PieceCode::new(us, Piece::Rook), rook_from);
            self.place_piece(us, Piece::Rook, rook_to);
            self.zkey
                .toggle_piece(PieceCode::new(us, Piece::Rook), rook_to);
        }

        // Pawn move resets halfmove clock
        if piece == Piece::Pawn {
            self.halfmove_clock = 0;

            // If move was a double pawn move, update ep square
            if mv.is_double_push() {
                let ep_square = Square::new((from.u8() + to.u8()) >> 1);

                // Update the zkey only if the double pushed pawn has an opposition pawn next to it
                if ep_hashable(&self.mailbox, ep_square, them) {
                    self.zkey.toggle_ep_file(ep_square);
                }

                self.ep_square = ep_square;
            }
        }

        self.side_to_move = them;
        self.zkey.toggle_side();

        self.update_pins(Colour::White);
        self.update_pins(Colour::Black);
        self.update_checkers();
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

        self.update_pins(Colour::White);
        self.update_pins(Colour::Black);
        self.update_checkers();
    }

    /// Flip the side to move without performing any move on the board
    #[inline(always)]
    pub fn make_null_move(&mut self, state: &mut StateInfo) {
        debug_assert!(self.checkers.is_empty());

        state.zkey = self.zkey;
        state.ep_square = self.ep_square;
        state.halfmove_clock = self.halfmove_clock;
        state.fullmove_counter = self.fullmove_counter;

        self.halfmove_clock += 1;
        if self.side_to_move == Colour::Black {
            self.fullmove_counter += 1;
        }

        // Unhash old en passant file if it exists
        if ep_hashable(&self.mailbox, self.ep_square, self.side_to_move) {
            self.zkey.toggle_ep_file(self.ep_square);
        }
        self.ep_square = Square::NONE;

        self.side_to_move = self.side_to_move.opposite();
        self.zkey.toggle_side();

        self.update_checkers();
    }

    #[inline(always)]
    pub fn undo_null_move(&mut self, prev: &StateInfo) {
        self.zkey = prev.zkey;
        self.halfmove_clock = prev.halfmove_clock;
        self.fullmove_counter = prev.fullmove_counter;
        self.ep_square = prev.ep_square;
        self.side_to_move = self.side_to_move.opposite();

        self.update_checkers();
    }

    /// Return a bitboard of all non-pawn pieces (excluding king) for the provided side
    #[inline(always)]
    pub fn non_pawn_material(&self, c: Colour) -> Bitboard {
        let c_pieces = self.pieces[c.idx()];
        c_pieces[Piece::Queen.idx()]
            | c_pieces[Piece::Rook.idx()]
            | c_pieces[Piece::Bishop.idx()]
            | c_pieces[Piece::Knight.idx()]
    }

    #[inline(always)]
    pub fn can_castle(&self, c: Castling) -> bool {
        self.castling_rights.contains(c)
    }

    #[inline(always)]
    pub fn castling_impeded(&self, c: Castling) -> bool {
        let path = CASTLING_PATHS[c.bits() as usize];
        self.occupancy[2] & path != Bitboard::new(0)
    }

    /// Calculate the pinned pieces for the specified side and the pinning piececs for the opponent
    #[inline(always)]
    pub fn update_pins(&mut self, c: Colour) {
        let bbs = bitboards();

        let king_sq = self.king_square(c);

        self.pinned_pieces[c.idx()] = Bitboard::new(0);
        self.pinners[c.opposite().idx()] = Bitboard::new(0);

        // A sniper is a slider which attacks the king when a piece and other snipers are removed
        let mut snipers = (bbs.bishop_attacks(king_sq, Bitboard::new(0))
            & self.bishop_sliders(c.opposite()))
            | (bbs.rook_attacks(king_sq, Bitboard::new(0)) & self.rook_sliders(c.opposite()));
        let occ = self.occupancy[2] ^ snipers;

        while !snipers.is_empty() {
            let sniper_sq = snipers.pop_lsb();
            // Occupied squares between the king and sniper
            let b = bbs.evasion_mask(king_sq, sniper_sq) & occ;

            if b.bit_count() == 1 {
                self.pinned_pieces[c.idx()] |= b;
                // The pinned piece is ours, so add sniper to pinners
                if !(b & self.occupancy[c.idx()]).is_empty() {
                    self.pinners[c.opposite().idx()] |= sniper_sq.bitboard();
                }
            }
        }
    }

    #[inline(always)]
    pub fn pinned_pieces(&self, c: Colour) -> Bitboard {
        self.pinned_pieces[c.idx()]
    }

    /// Find all checking pieces to the current sides king
    #[inline(always)]
    pub fn update_checkers(&mut self) {
        let us = self.side_to_move;
        let occ = self.occupancy[2];
        self.checkers = self.attackers_to(self.king_square(us), occ);
    }

    /// Compute a bitboard of all opponent attackers to a square
    #[inline(always)]
    pub fn attackers_to(&self, sq: Square, occ: Bitboard) -> Bitboard {
        let bbs = bitboards();

        let us = self.side_to_move;
        let them = us.opposite();

        return (bbs.bishop_attacks(sq, occ) & self.bishop_sliders(them))
            | (bbs.rook_attacks(sq, occ) & self.rook_sliders(them))
            | bbs.knight_attacks(sq) & self.pieces[them.idx()][Piece::Knight.idx()]
            | bbs.pawn_attacks(sq, us) & self.pieces[them.idx()][Piece::Pawn.idx()]
            | bbs.king_attacks(sq) & self.pieces[them.idx()][Piece::King.idx()];
    }

    /// Check if a square is attacked by the opponent
    #[inline(always)]
    pub fn attackers_to_exist(&self, sq: Square, occ: Bitboard) -> bool {
        let bbs = bitboards();

        let us = self.side_to_move;
        let them = us.opposite();

        return !(bbs.bishop_attacks(sq, occ) & self.bishop_sliders(them)).is_empty()
            || !(bbs.rook_attacks(sq, occ) & self.rook_sliders(them)).is_empty()
            || !(bbs.knight_attacks(sq) & self.pieces[them.idx()][Piece::Knight.idx()]).is_empty()
            || !(bbs.pawn_attacks(sq, us) & self.pieces[them.idx()][Piece::Pawn.idx()]).is_empty()
            || !(bbs.king_attacks(sq) & self.pieces[them.idx()][Piece::King.idx()]).is_empty();
    }

    /// Checks if a move is pseudo-legal in the current position. 
    /// 
    /// This is used to validate singleton search moves such as PV, TT, 
    /// and killer moves before they bypass normal move generation.
    ///
    /// The move is not assumed to have come from the current position, so we
    /// check piece ownership, occupancy constraints, and piece-specific move 
    /// geometry, but it does not check whether the move leaves the king in check.
    #[inline(always)]
    pub fn is_pseudo_legal(&self, mv: Move) -> bool {
        let bbs = bitboards();

        let us = self.side_to_move;
        let them = us.opposite();
        let from = mv.from();
        let to = mv.to();

        let opp_occ = self.occupancy[them.idx()];
        let all_occ = self.occupancy[2];

        // Check the moving piece actually exists
        let Some(piece) = self.mailbox.piece_at(from) else {
            return false;
        };

        // Check the moving piece is the correct side
        if (self.occupancy[us.idx()] & from.bitboard()).is_empty() {
            return false;
        }

        // 'to' square cannot be occupied by a friendly piece
        if !(self.occupancy[us.idx()] & to.bitboard()).is_empty() {
            return false;
        }

        // If the move is a capture, 'to' square much be occupied by an enemy
        // We skip pawns here as en passant doesn't follow this behaviour.
        // Similarly, a non-capture shouldn't land on an enemy piece
        if piece != Piece::Pawn {
            if mv.is_capture() && (to.bitboard() & opp_occ).is_empty() {
                return false;
            }
            if !mv.is_capture() && !(to.bitboard() & opp_occ).is_empty() {
                return false;
            }
        }

        match piece {
            Piece::Pawn => {
                if mv.is_ep_capture() {
                    return to == self.ep_square;
                }

                if mv.is_capture() {
                    return !(bbs.pawn_attacks(from, us) & to.bitboard() & opp_occ).is_empty();
                }

                // Regular non-capturing promotion
                if mv.is_promotion() {
                    return (to.bitboard() & opp_occ).is_empty();
                }

                if mv.is_double_push() {
                    let middle = Square::new((to.u8() + from.u8()) >> 1);
                    return ((middle.bitboard() | to.bitboard()) & all_occ).is_empty();
                }

                // Single push
                let delta = to.u8() as i32 - from.u8() as i32;
                let expected_delta = if us == Colour::White { 8 } else { -8 };

                return (to.bitboard() & opp_occ).is_empty() && delta == expected_delta;
            }
            Piece::Knight => {
                return !(bbs.knight_attacks(from) & to.bitboard()).is_empty();
            }
            Piece::Bishop => {
                let attack = bbs.bishop_attacks(from, all_occ);
                return !(attack & to.bitboard()).is_empty();
            }
            Piece::Rook => {
                let attack = bbs.rook_attacks(from, all_occ);
                return !(attack & to.bitboard()).is_empty();
            }
            Piece::Queen => {
                let attack = bbs.bishop_attacks(from, all_occ) | bbs.rook_attacks(from, all_occ);
                return !(attack & to.bitboard()).is_empty();
            }
            Piece::King => {
                let in_check = !self.checkers.is_empty();

                if let Some(castle_type) = mv.castle_type() {
                    if in_check {
                        return false;
                    }

                    let castling_flag = match (us, castle_type) {
                        (Colour::White, CastlingType::Kingside) => Castling::WHITE_OO,
                        (Colour::White, CastlingType::Queenside) => Castling::WHITE_OOO,
                        (Colour::Black, CastlingType::Kingside) => Castling::BLACK_OO,
                        (Colour::Black, CastlingType::Queenside) => Castling::BLACK_OOO,
                    };

                    return self.can_castle(castling_flag) && !self.castling_impeded(castling_flag);
                }

                let mask = bbs.king_attacks(from);
                return !(to.bitboard() & mask).is_empty();
            }
        }
    }

    /// Check if a pseudo-legal move is legal or not \
    /// Only checks if the king is in check after the move, not before
    #[inline(always)]
    pub fn is_legal(&self, mv: Move) -> bool {
        let bbs = bitboards();

        let us = self.side_to_move;
        let them = us.opposite();
        let from = mv.from();
        let to = mv.to();

        // En passant move should not leave the king in check
        if mv.is_ep_capture() {
            #[rustfmt::skip]
            let ep_offset = if us == Colour::White { Direction::North } else { Direction::South };
            let ep_pawn_sq = Square::new((to.u8() as i8 - ep_offset as i8) as u8);
            let occ = (self.occupancy[2] ^ from.bitboard() ^ ep_pawn_sq.bitboard()) | to.bitboard();

            return (bbs.bishop_attacks(self.king_square(us), occ) & self.bishop_sliders(them))
                .is_empty()
                && (bbs.rook_attacks(self.king_square(us), occ) & self.rook_sliders(them))
                    .is_empty();
        }

        // Castling does not move through check or leave the king in check
        if mv.is_castle() {
            let step = if to.u8() > from.u8() {
                Direction::West
            } else {
                Direction::East
            };
            let mut sq = to.u8() as i32;
            let end = from.u8() as i32;

            while sq != end {
                if self.attackers_to_exist(Square::new(sq as u8), self.occupancy[2]) {
                    return false;
                }
                sq += step as i32;
            }
        }

        // If the moving piece is a king, it should not leave the king in check
        if mv.from() == self.king_square(us) {
            return !self.attackers_to_exist(to, self.occupancy[2] ^ from.bitboard());
        }

        // A non-king move is legal iff it is not pinned or it is moving along the pin
        return (self.pinned_pieces(us) & from.bitboard()).is_empty()
            || !(bbs.line_bb(from, to) & self.king_square(us).bitboard()).is_empty();
    }

    #[inline(always)]
    pub fn insufficient_material(&self) -> bool {
        let white = self.pieces[Colour::White.idx()];
        let black = self.pieces[Colour::Black.idx()];

        // Sufficient material to continue
        if !(white[Piece::Pawn.idx()]
            | black[Piece::Pawn.idx()]
            | white[Piece::Rook.idx()]
            | black[Piece::Rook.idx()]
            | white[Piece::Queen.idx()]
            | black[Piece::Queen.idx()])
        .is_empty()
        {
            return false;
        }

        let white_knights = white[Piece::Knight.idx()].bit_count();
        let black_knights = black[Piece::Knight.idx()].bit_count();
        let white_bishops = white[Piece::Bishop.idx()].bit_count();
        let black_bishops = black[Piece::Bishop.idx()].bit_count();

        let white_minors = white_knights + white_bishops;
        let black_minors = black_knights + black_bishops;
        let minor_pieces = white_minors + black_minors;

        // We only need to do a detailed check for insufficient material
        // in the case of two minor pieces, as other cases are trivial
        if minor_pieces < 2 {
            return true;
        }
        if minor_pieces > 2 {
            return false;
        }

        // Sufficient material if each side has an opposite coloured bishop
        if white_bishops == 1 && black_bishops == 1 {
            let wb_square = white[Piece::Bishop.idx()].lsb();
            let bb_square = black[Piece::Bishop.idx()].lsb();

            return wb_square.colour() == bb_square.colour();
        }

        // Sufficient material only if one side has opposite coloured bishops
        if white_bishops == 2 || black_bishops == 2 {
            let mut bishops_bb = match white_bishops {
                0 => black[Piece::Bishop.idx()],
                2 => white[Piece::Bishop.idx()],
                _ => unreachable!(),
            };

            let bishop1 = bishops_bb.pop_lsb();
            let bishop2 = bishops_bb.pop_lsb();

            return bishop1.colour() == bishop2.colour();
        }

        false
    }

    #[inline(always)]
    pub fn captured_piece(&self, mv: Move) -> Option<Piece> {
        if mv.is_ep_capture() {
            Some(Piece::Pawn)
        } else {
            self.mailbox.piece_at(mv.to())
        }
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
        let mut rights = Castling::NONE;
        if fen_parts[2] != "-" {
            for c in fen_parts[2].chars() {
                match c {
                    'K' => rights |= Castling::WHITE_OO,
                    'Q' => rights |= Castling::WHITE_OOO,
                    'k' => rights |= Castling::BLACK_OO,
                    'q' => rights |= Castling::BLACK_OOO,
                    _ => unreachable!(),
                }
            }
        }
        position.castling_rights = rights;

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

        position.update_pins(Colour::White);
        position.update_pins(Colour::Black);
        position.update_checkers();

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
    use crate::{
        movegen::{MoveList, generate_all, generate_evasions},
        types::MoveFlag,
    };

    use super::*;

    struct GoalState {
        ep_square: Square,
        castling_rights: Castling,
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
        assert_eq!(state.castling_rights, goal.castling_rights);
        assert_eq!(state.captured_piece, None);
        assert_eq!(state.halfmove_clock, goal.halfmove_clock);
        assert_eq!(state.fullmove_counter, goal.fullmove_counter);
    }

    fn check_position_correctness(actual: &Position, expected: &Position) {
        assert_eq!(actual.pieces, expected.pieces);
        assert_eq!(actual.occupancy, expected.occupancy);
        assert_eq!(actual.mailbox, expected.mailbox);
        assert_eq!(actual.pinned_pieces, expected.pinned_pieces);
        assert_eq!(actual.pinners, expected.pinners);

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
            castling_rights: Castling::DEFAULT,
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
            castling_rights: Castling::WHITE_CASTLING,
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
        let mut expected_pos = Position::from_fen(expected_fen);
        expected_pos.zkey = pos.zkey;
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
        let mut expected_pos = Position::from_fen(expected_fen);
        expected_pos.zkey = pos.zkey;
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
            let mut expected_pos = Position::from_fen(case.expected_fen);
            expected_pos.zkey = pos.zkey;

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

    #[test]
    fn position_make_null_move_is_correct() {
        let pos = Position::from_fen("4k3/5n2/8/2Pp3B/1b6/8/3R4/4K3 w - d6 0 1");
        let mut key = pos.zkey;
        let mut mut_pos = pos;

        let mut state = StateInfo::new();
        state.set_from_position(&mut_pos);
        mut_pos.make_null_move(&mut state);

        // Check null move only changes exactly what is required
        assert_eq!(mut_pos.ep_square, Square::NONE);
        assert_eq!(mut_pos.side_to_move, Colour::Black);
        key.toggle_side();
        key.toggle_ep_file(Square::D6);
        assert_eq!(mut_pos.zkey, key);
        assert_eq!(pos.halfmove_clock + 1, mut_pos.halfmove_clock);
        assert_eq!(pos.checkers, mut_pos.checkers); // Both positions have no checkers, so both empty

        // Null move leaves everything else unchanged
        assert_eq!(pos.occupancy, mut_pos.occupancy);
        assert_eq!(pos.pieces, mut_pos.pieces);
        assert_eq!(pos.mailbox, mut_pos.mailbox);
        assert_eq!(pos.castling_rights, mut_pos.castling_rights);
    }

    #[test]
    fn position_undo_null_move_is_correct() {
        let pos = Position::from_fen("4k3/5n2/8/2Pp3B/1b6/8/3R4/4K3 w - d6 0 1");
        let mut mut_pos = pos;

        let mut state = StateInfo::new();
        state.set_from_position(&pos);
        mut_pos.make_null_move(&mut state);
        mut_pos.undo_null_move(&state);

        assert_eq!(mut_pos, pos);
    }

    #[test]
    fn position_can_castle_is_correct() {
        let pos = Position::default();
        assert!(pos.can_castle(Castling::WHITE_OO));
        assert!(pos.can_castle(Castling::WHITE_OOO));
        assert!(pos.can_castle(Castling::BLACK_OO));
        assert!(pos.can_castle(Castling::BLACK_OOO));

        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1");
        assert_eq!(pos.can_castle(Castling::WHITE_OO), false);
        assert_eq!(pos.can_castle(Castling::WHITE_OOO), false);
        assert_eq!(pos.can_castle(Castling::BLACK_OO), false);
        assert_eq!(pos.can_castle(Castling::BLACK_OOO), false);
    }

    #[test]
    fn position_castling_impeded_is_correct() {
        let pos = Position::default();
        assert!(pos.castling_impeded(Castling::WHITE_OO));
        assert!(pos.castling_impeded(Castling::WHITE_OOO));
        assert!(pos.castling_impeded(Castling::BLACK_OO));
        assert!(pos.castling_impeded(Castling::BLACK_OOO));

        let pos = Position::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
        assert_eq!(pos.castling_impeded(Castling::WHITE_OO), false);
        assert_eq!(pos.castling_impeded(Castling::WHITE_OOO), false);
        assert_eq!(pos.castling_impeded(Castling::BLACK_OO), false);
        assert_eq!(pos.castling_impeded(Castling::BLACK_OOO), false);
    }

    #[test]
    fn is_pseudo_legal_move_is_consistent() {
        let pos = Position::default();
        let mut moves = MoveList::new();
        generate_all(&pos, &mut moves);

        // Check all pseudo-legal moves pass
        for &mv in moves.as_slice() {
            assert!(pos.is_pseudo_legal(mv));
        }

        // Correctly classes evasion moves
        let pos =
            Position::from_fen("rnbqk1nr/ppp2ppp/4p3/3p4/2PP4/2b2N2/PP2P1PP/R1BQKB1R w KQkq - 0 1");
        generate_evasions(&pos, &mut moves);

        let mut count = 0;
        for &mv in moves.as_slice() {
            assert!(pos.is_pseudo_legal(mv));
            count += 1;
        }

        assert_eq!(count, 5);
    }

    #[test]
    fn is_pseudo_legal_on_pawn_moves() {
        // Correct singe/double push filtering
        let pos =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/P1p5/1P1p4/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let valid1 = Move::new(Square::A2, Square::A3, MoveFlag::Quiet);
        let valid2 = Move::new(Square::C2, Square::C3, MoveFlag::Quiet);
        let valid3 = Move::new(Square::E2, Square::E3, MoveFlag::Quiet);
        let valid4 = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);
        let blocked1 = Move::new(Square::A2, Square::A4, MoveFlag::DoublePush);
        let blocked2 = Move::new(Square::B2, Square::B4, MoveFlag::DoublePush);
        let blocked3 = Move::new(Square::C2, Square::C4, MoveFlag::DoublePush);
        let blocked4 = Move::new(Square::D2, Square::D4, MoveFlag::DoublePush);
        let blocked5 = Move::new(Square::B2, Square::B3, MoveFlag::Quiet);
        let blocked6 = Move::new(Square::D2, Square::D3, MoveFlag::Quiet);

        assert!(pos.is_pseudo_legal(valid1));
        assert!(pos.is_pseudo_legal(valid2));
        assert!(pos.is_pseudo_legal(valid3));
        assert!(pos.is_pseudo_legal(valid4));
        assert_eq!(pos.is_pseudo_legal(blocked1), false);
        assert_eq!(pos.is_pseudo_legal(blocked2), false);
        assert_eq!(pos.is_pseudo_legal(blocked3), false);
        assert_eq!(pos.is_pseudo_legal(blocked4), false);
        assert_eq!(pos.is_pseudo_legal(blocked5), false);
        assert_eq!(pos.is_pseudo_legal(blocked6), false);

        // Correctly validates en passant moves
        let pos =
            Position::from_fen("rnbqkbnr/1p2pppp/p7/1PppP3/8/8/PPPPPPPP/RNBQKBNR w KQkq d6 0 1");
        let valid_ep = Move::new(Square::E5, Square::D6, MoveFlag::EpCapture);
        let invalid_ep1 = Move::new(Square::B5, Square::C6, MoveFlag::EpCapture);
        let invalid_ep2 = Move::new(Square::B5, Square::A6, MoveFlag::EpCapture);

        assert!(pos.is_pseudo_legal(valid_ep));
        assert_eq!(pos.is_pseudo_legal(invalid_ep1), false);
        assert_eq!(pos.is_pseudo_legal(invalid_ep2), false);
    }

    #[test]
    fn is_pseudo_legal_on_sliders() {
        // Test blocked vs open bishop diagonals
        let open = Position::from_fen("3k4/8/8/8/8/3B4/8/3K4 w - - 0 1");
        let blocked = Position::from_fen("3k4/8/6p1/1P6/8/3B4/8/3K4 w - - 0 1");

        let blocked1 = Move::new(Square::D3, Square::A6, MoveFlag::Quiet);
        let blocked2 = Move::new(Square::D3, Square::B5, MoveFlag::Quiet);
        let blocked3 = Move::new(Square::D3, Square::H7, MoveFlag::Quiet);
        let capture1 = Move::new(Square::D3, Square::G6, MoveFlag::Capture);

        assert!(open.is_pseudo_legal(blocked1));
        assert!(open.is_pseudo_legal(blocked2));
        assert!(open.is_pseudo_legal(blocked3));
        assert!(!open.is_pseudo_legal(capture1)); // Capture invalid

        assert!(!blocked.is_pseudo_legal(blocked1));
        assert!(!blocked.is_pseudo_legal(blocked2));
        assert!(!blocked.is_pseudo_legal(blocked3));
        assert!(blocked.is_pseudo_legal(capture1)); // Capture valid

        // Test blocked vs open rook file/ranks
        let open = Position::from_fen("3k4/8/8/8/4R3/8/8/3K4 w - - 0 1");
        let blocked = Position::from_fen("3k4/8/4p3/8/4R1P1/8/8/3K4 w - - 0 1");

        let blocked1 = Move::new(Square::E4, Square::G4, MoveFlag::Quiet);
        let blocked2 = Move::new(Square::E4, Square::H4, MoveFlag::Quiet);
        let blocked3 = Move::new(Square::E4, Square::E7, MoveFlag::Quiet);
        let capture1 = Move::new(Square::E4, Square::E6, MoveFlag::Capture);

        assert!(open.is_pseudo_legal(blocked1));
        assert!(open.is_pseudo_legal(blocked2));
        assert!(open.is_pseudo_legal(blocked3));
        assert!(!open.is_pseudo_legal(capture1)); // Capture invalid

        assert!(!blocked.is_pseudo_legal(blocked1));
        assert!(!blocked.is_pseudo_legal(blocked2));
        assert!(!blocked.is_pseudo_legal(blocked3));
        assert!(blocked.is_pseudo_legal(capture1)); // Capture valid
    }

    #[test]
    fn is_pseudo_legal_castling() {
        let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

        // Allows legal castles
        let white_oo = Move::new(Square::E1, Square::G1, MoveFlag::KingCastle);
        let white_ooo = Move::new(Square::E1, Square::C1, MoveFlag::QueenCastle);

        assert!(pos.is_pseudo_legal(white_oo));
        assert!(pos.is_pseudo_legal(white_ooo));

        // Disallow castles when rights are missing, path blocked, or in check
        let mut pos = Position::from_fen("r3k2r/pppp1ppp/8/8/8/4R3/PPPPPPPP/RN2K2R w Qkq - 0 1");
        let mut state = StateInfo::new();
        let no_rights = Move::new(Square::E1, Square::G1, MoveFlag::KingCastle);
        let blocked = Move::new(Square::E1, Square::C1, MoveFlag::QueenCastle);
        let in_check1 = Move::new(Square::E8, Square::G8, MoveFlag::KingCastle);
        let in_check2 = Move::new(Square::E8, Square::C8, MoveFlag::QueenCastle);

        assert_eq!(pos.is_pseudo_legal(no_rights), false);
        assert_eq!(pos.is_pseudo_legal(blocked), false);
        pos.make_null_move(&mut state);
        assert_eq!(pos.is_pseudo_legal(in_check1), false);
        assert_eq!(pos.is_pseudo_legal(in_check2), false);
    }

    #[test]
    fn position_update_pins_is_correct() {
        let w = Colour::White;
        let b = Colour::Black;
        let mut pos = Position::default();

        // No pinned/pinners in starting position
        pos.update_pins(w);
        pos.update_pins(b);

        assert_eq!(pos.pinned_pieces[w.idx()], Bitboard::new(0));
        assert_eq!(pos.pinned_pieces[b.idx()], Bitboard::new(0));
        assert_eq!(pos.pinners[w.idx()], Bitboard::new(0));
        assert_eq!(pos.pinners[b.idx()], Bitboard::new(0));

        // Correctly updates the pinned/pinners
        let mut pos = Position::from_fen("4k3/3np3/3q2P1/1B1r3B/Q2nR1b1/5P2/8/3KN2q w - - 0 1");
        pos.update_pins(w);
        pos.update_pins(b);

        let mut w_pinned = Bitboard::new(0);
        w_pinned.set_square(Square::E1);
        w_pinned.set_square(Square::F3);
        w_pinned.set_square(Square::D4);

        let mut b_pinned = Bitboard::new(0);
        b_pinned.set_square(Square::G6);
        b_pinned.set_square(Square::D7);
        b_pinned.set_square(Square::E7);

        let mut w_pinners = Bitboard::new(0);
        w_pinners.set_square(Square::H1);
        w_pinners.set_square(Square::G4);

        let mut b_pinners = Bitboard::new(0);
        b_pinners.set_square(Square::E4);
        b_pinners.set_square(Square::B5);
        b_pinners.set_square(Square::A4);

        assert_eq!(pos.pinned_pieces[w.idx()], w_pinned);
        assert_eq!(pos.pinners[b.idx()], w_pinners);
        assert_eq!(pos.pinned_pieces[b.idx()], b_pinned);
        assert_eq!(pos.pinners[w.idx()], b_pinners);
    }

    #[test]
    fn position_update_checkers_is_correct() {
        let mut pos = Position::default();

        // No checkers in default position
        assert_eq!(pos.checkers, Bitboard::new(0));

        // Gets all possible checkers for white
        let mut expected = Bitboard::new(0);
        expected.set_square(Square::B1);
        expected.set_square(Square::G1);
        expected.set_square(Square::D2);
        expected.set_square(Square::E2);
        expected.set_square(Square::E3);
        expected.set_square(Square::A4);

        pos = Position::from_fen("8/8/8/8/b7/4n3/3kp3/1q1K2r1 w - - 0 1");
        assert_eq!(pos.checkers, expected);

        // Gets all possible checkers for black
        let mut expected = Bitboard::new(0);
        expected.set_square(Square::D2);
        expected.set_square(Square::H4);
        expected.set_square(Square::C7);
        expected.set_square(Square::F7);
        expected.set_square(Square::B8);
        expected.set_square(Square::E8);

        pos = Position::from_fen("1Q1kK3/2P2N2/8/8/7B/8/3R4/5K2 b - - 0 1");
        pos.update_checkers();
        assert_eq!(pos.checkers, expected);
    }

    fn positions_have_sufficient_material(positions: &[Position]) {
        for &pos in positions {
            assert_eq!(pos.insufficient_material(), false);
        }
    }

    fn positions_have_insufficient_material(positions: &[Position]) {
        for &pos in positions {
            assert!(pos.insufficient_material());
        }
    }

    #[test]
    fn position_insufficient_material_is_correct() {
        // One bishop/knight is insufficient
        let pos1 = Position::from_fen("3k4/5b2/8/8/8/8/8/3K4 w - - 0 1");
        let pos2 = Position::from_fen("3k4/8/8/8/8/5N2/8/3K4 b - - 0 1");
        positions_have_insufficient_material(&[pos1, pos2]);

        // More than two bishops/knights is sufficient
        let pos1 = Position::from_fen("3k4/4b3/8/8/8/4BN2/8/3K4 w - - 0 1");
        let pos2 = Position::from_fen("3k4/4b3/5b2/8/8/1N6/8/3K4 b - - 0 1");
        positions_have_sufficient_material(&[pos1, pos2]);

        // One pawn, rook, or queen is sufficient
        let pos1 = Position::from_fen("3k4/8/8/8/8/8/5R2/3K4 w - - 0 1");
        let pos2 = Position::from_fen("3k4/8/8/8/8/8/5Q2/3K4 b - - 0 1");
        let pos3 = Position::from_fen("3k4/8/8/8/8/8/5P2/3K4 w - - 0 1");
        positions_have_sufficient_material(&[pos1, pos2, pos3]);

        // Opposite coloured bishops are always sufficient
        let pos1 = Position::from_fen("3k4/8/8/8/8/8/4BB2/3K4 w - - 0 1");
        let pos2 = Position::from_fen("3k4/4bb2/8/8/8/8/8/3K4 b - - 0 1");
        let pos3 = Position::from_fen("3k4/5b2/8/8/8/8/5B2/3K4 b - - 0 1");
        positions_have_sufficient_material(&[pos1, pos2, pos3]);

        // Two knights are always sufficient
        let pos1 = Position::from_fen("3k4/8/8/8/8/3NN3/8/3K4 b - - 0 1");
        let pos2 = Position::from_fen("3k4/4nn2/8/8/8/8/8/3K4 w - - 0 1");
        let pos3 = Position::from_fen("3k4/5n2/8/8/8/3N4/8/3K4 w - - 0 1");
        positions_have_sufficient_material(&[pos1, pos2, pos3]);

        // Bishop + knight is always sufficient
        let pos1 = Position::from_fen("3k4/8/8/8/8/3NB3/8/3K4 w - - 0 1");
        let pos2 = Position::from_fen("3k4/5n2/8/1b6/8/8/8/3K4 w - - 0 1");
        let pos3 = Position::from_fen("3k4/4n3/8/8/8/4B3/8/3K4 w - - 0 1");
        positions_have_sufficient_material(&[pos1, pos2, pos3]);

        // Same coloured bishops are insufficient
        let pos1 = Position::from_fen("3k4/8/8/8/8/3B4/4B3/3K4 w - - 0 1");
        let pos2 = Position::from_fen("3k4/4b3/5b2/8/8/8/8/3K4 w - - 0 1");
        let pos3 = Position::from_fen("3k4/4b3/8/8/8/8/5B2/3K4 w - - 0 1");
        positions_have_insufficient_material(&[pos1, pos2, pos3]);
    }
}
