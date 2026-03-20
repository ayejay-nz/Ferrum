use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

use crate::{
    movegen::{MoveList, generate_legal},
    position::Position,
    types::{Move, Piece, Square},
};

const ENTRY_SIZE: u64 = 16;

fn book_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../books")
        .join("Cerebellum3Merge.bin")
}

fn promo_piece(code: u16) -> Option<Option<Piece>> {
    match code {
        0 => Some(None),
        1 => Some(Some(Piece::Knight)),
        2 => Some(Some(Piece::Bishop)),
        3 => Some(Some(Piece::Rook)),
        4 => Some(Some(Piece::Queen)),
        _ => None,
    }
}

fn decode_book_move(book_move: u16) -> Option<(Square, Square, Option<Piece>)> {
    let to_file = (book_move & 0x7) as u8;
    let to_rank = ((book_move >> 3) & 0x7) as u8;
    let from_file = ((book_move >> 6) & 0x7) as u8;
    let from_rank = ((book_move >> 9) & 0x7) as u8;
    let promo = promo_piece((book_move >> 12) & 0x7)?;

    let from = Square::from_coords(from_rank, from_file);
    let mut to = Square::from_coords(to_rank, to_file);

    // Polyglot castling is encoded as king captures rook.
    match (from, to) {
        (Square::E1, Square::H1) => to = Square::G1,
        (Square::E1, Square::A1) => to = Square::C1,
        (Square::E8, Square::H8) => to = Square::G8,
        (Square::E8, Square::A8) => to = Square::C8,
        _ => {}
    }

    Some((from, to, promo))
}

pub fn probe_opening_book(pos: &Position) -> Option<Move> {
    let path = book_path();
    let mut file = File::open(path).ok()?;

    let key = pos.zkey.0;
    let n_entries = file.metadata().ok()?.len() / ENTRY_SIZE;

    let mut lo = 0;
    let mut hi = n_entries;

    while lo < hi {
        let mid = (lo + hi) / 2;
        file.seek(SeekFrom::Start(mid * ENTRY_SIZE)).ok()?;

        let mut buf = [0u8; 16];
        file.read_exact(&mut buf).ok()?;

        let entry_key = u64::from_be_bytes(buf[0..8].try_into().ok()?);

        if entry_key < key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }

    let legal_moves = generate_legal(pos, &mut MoveList::new());

    let mut best: Option<(Move, u16)> = None;
    let mut i = lo;

    while i < n_entries {
        file.seek(SeekFrom::Start(i * ENTRY_SIZE)).ok()?;

        let mut buf = [0u8; 16];
        file.read_exact(&mut buf).ok()?;

        let entry_key = u64::from_be_bytes(buf[0..8].try_into().ok()?);
        if entry_key != key {
            break;
        }

        let raw_move = u16::from_be_bytes(buf[8..10].try_into().ok()?);
        let weight = u16::from_be_bytes(buf[10..12].try_into().ok()?);

        let (from, to, promo) = decode_book_move(raw_move)?;

        if let Some(mv) = legal_moves
            .as_slice()
            .iter()
            .copied()
            .find(|&mv| mv.from() == from && mv.to() == to && mv.promotion_piece() == promo)
        {
            if best.is_none_or(|(_, best_weight)| weight > best_weight) {
                best = Some((mv, weight));
            }
        }

        i += 1;
    }

    best.map(|(mv, _)| mv)
}
