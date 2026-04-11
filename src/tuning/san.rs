use std::io;

use crate::{
    movegen::{MoveList, generate_legal},
    position::Position,
    types::{CastlingType, Move, Piece, Square},
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct SanSpec {
    piece: Piece,
    to: Square,
    capture: bool,
    promotion: Option<Piece>,
    from_file: Option<u8>,
    from_rank: Option<u8>,
}

pub fn san_to_move(pos: &Position, san: &str) -> io::Result<Move> {
    let san = san.trim_end_matches(['+', '#']);

    if matches!(san, "O-O" | "0-0") {
        return find_castle(pos, CastlingType::Kingside, san);
    }

    if matches!(san, "O-O-O" | "0-0-0") {
        return find_castle(pos, CastlingType::Queenside, san);
    }

    let spec = parse_san_spec(san)?;
    let mut moves = MoveList::new();
    let legal = generate_legal(pos, &mut moves);

    let matches: Vec<Move> = legal
        .as_slice()
        .iter()
        .copied()
        .filter(|&mv| {
            let from = mv.from();
            let Some(piece) = pos.mailbox.piece_at(from) else {
                return false;
            };

            piece == spec.piece
                && mv.to() == spec.to
                && mv.is_capture() == spec.capture
                && mv.promotion_piece() == spec.promotion
                && spec.from_file.is_none_or(|f| from.file() == f)
                && spec.from_rank.is_none_or(|r| from.rank() == r)
        })
        .collect();

    match matches.as_slice() {
        [mv] => Ok(*mv),
        [] => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("no legal move matches SAN: {san}"),
        )),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("SAN is still ambiguous after legal filtering: {san}"),
        )),
    }
}

fn find_castle(pos: &Position, side: CastlingType, san: &str) -> io::Result<Move> {
    let mut moves = MoveList::new();
    let legal = generate_legal(pos, &mut moves);

    legal
        .as_slice()
        .iter()
        .copied()
        .find(|mv| mv.castle_type() == Some(side))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("illegal SAN: {san}")))
}

fn parse_san_spec(san: &str) -> io::Result<SanSpec> {
    let (body, promotion) = if let Some((lhs, rhs)) = san.split_once('=') {
        (lhs, Some(parse_piece_letter(rhs.as_bytes()[0], true)?))
    } else {
        (san, None)
    };

    if body.len() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("bad SAN: {san}"),
        ));
    }

    let to = parse_square(&body[body.len() - 2..])?;
    let prefix = &body[..body.len() - 2];
    let capture = prefix.contains('x');
    let prefix = prefix.replace('x', "");

    let bytes = prefix.as_bytes();
    let (piece, disamb) = match bytes.first().copied() {
        Some(b'N' | b'B' | b'R' | b'Q' | b'K') => {
            (parse_piece_letter(bytes[0], false)?, &bytes[1..])
        }
        _ => (Piece::Pawn, bytes),
    };

    let mut from_file = None;
    let mut from_rank = None;

    for &b in disamb {
        match b {
            b'a'..=b'h' => from_file = Some(b - b'a'),
            b'1'..=b'8' => from_rank = Some(b - b'1'),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("bad SAN disambiguation: {san}"),
                ));
            }
        }
    }

    Ok(SanSpec {
        piece,
        to,
        capture,
        promotion,
        from_file,
        from_rank,
    })
}

fn parse_square(s: &str) -> io::Result<Square> {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("bad square: {s}"),
        ));
    }

    let file = bytes[0];
    let rank = bytes[1];

    if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("bad square: {s}"),
        ));
    }

    Ok(Square::from_coords(rank - b'1', file - b'a'))
}

fn parse_piece_letter(b: u8, allow_queen_only_promo: bool) -> io::Result<Piece> {
    let piece = match b {
        b'N' => Piece::Knight,
        b'B' => Piece::Bishop,
        b'R' => Piece::Rook,
        b'Q' => Piece::Queen,
        b'K' if !allow_queen_only_promo => Piece::King,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("bad piece letter: {}", b as char),
            ));
        }
    };

    Ok(piece)
}
