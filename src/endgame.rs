use crate::{
    bitboard::Bitboard,
    evaluate::{Eval, EvalInfo},
    params::Params,
    position::Position,
    types::{Colour, Piece, Square},
};

const SCALE_NORMAL: i32 = 128;

const RANK_1: u8 = 0;
const RANK_3: u8 = 2;
const RANK_4: u8 = 3;
const RANK_5: u8 = 4;
const RANK_6: u8 = 5;
const RANK_7: u8 = 6;
const RANK_8: u8 = 7;
const FILE_A: u8 = 0;
const FILE_D: u8 = 3;
const FILE_H: u8 = 7;

/// Map the square as if strong is white and strongs only pawn
/// is on the left half of the board
#[inline(always)]
fn normalise_krpkr(sq: Square, strong: Colour, mirror_file: bool) -> Square {
    let rank = if strong == Colour::White {
        sq.rank()
    } else {
        7 - sq.rank()
    };

    let file = if mirror_file {
        7 - sq.file()
    } else {
        sq.file()
    };

    Square::from_coords(rank, file)
}

#[inline(always)]
fn dist(a: Square, b: Square) -> i32 {
    a.file().abs_diff(b.file()).max(a.rank().abs_diff(b.rank())) as i32
}

#[inline(always)]
fn file_dist(a: Square, b: Square) -> i32 {
    a.file().abs_diff(b.file()) as i32
}

#[inline(always)]
fn north(sq: Square) -> Square {
    Square::from_coords(sq.rank() + 1, sq.file())
}

fn krpkr_scale(pos: &Position, info: &EvalInfo) -> Option<i32> {
    let strong = if info.pawns(Colour::White) == 1 {
        Colour::White
    } else {
        Colour::Black
    };
    let weak = strong.opposite();

    // Ensure we have a KRP vs KR endgame
    if info.queens(Colour::Both) != 0
        || info.minors(Colour::Both) != 0
        || info.rooks(strong) != 1
        || info.rooks(weak) != 1
        || info.pawns(strong) != 1
        || info.pawns(weak) != 0
    {
        return None;
    }

    let raw_wpsq = pos.pieces[strong.idx()][Piece::Pawn.idx()].lsb();
    let mirror_file = raw_wpsq.file() > FILE_D;

    let wksq = normalise_krpkr(pos.king_square(strong), strong, mirror_file);
    let bksq = normalise_krpkr(pos.king_square(weak), strong, mirror_file);
    let wrsq = normalise_krpkr(
        pos.pieces[strong.idx()][Piece::Rook.idx()].lsb(),
        strong,
        mirror_file,
    );
    let brsq = normalise_krpkr(
        pos.pieces[weak.idx()][Piece::Rook.idx()].lsb(),
        strong,
        mirror_file,
    );
    let wpsq = normalise_krpkr(raw_wpsq, strong, mirror_file);

    let file = wpsq.file();
    let rank = wpsq.rank();
    let queening_sq = Square::from_coords(RANK_8, file);
    let tempo = (pos.side_to_move == strong) as i32;

    // If the pawn is not too far advanced and the defending king defends the queening square,
    // use third rank defense
    if rank <= RANK_5
        && dist(bksq, queening_sq) <= 1
        && wksq.rank() <= RANK_5
        && (brsq.rank() == RANK_6 || (rank <= RANK_3 && wrsq.rank() != RANK_6))
    {
        return Some(0);
    }

    // The defending side saves a draw by checking from behind in case the pawn has advanced
    // to the 6th rank with the king behind
    if rank == RANK_6
        && dist(bksq, queening_sq) <= 1
        && wksq.rank() as i32 + tempo <= RANK_6 as i32
        && (brsq.rank() == RANK_1 || (tempo == 0 && file_dist(brsq, wpsq) >= 3))
    {
        return Some(0);
    }

    if rank >= RANK_6
        && bksq == queening_sq
        && brsq.rank() == RANK_1
        && (tempo == 0 || dist(wksq, wpsq) >= 2)
    {
        return Some(0);
    }

    // White pawn on a7 and rook on a8 is a draw if black's king is on g7 or h7 and the
    // black rook is behind the pawn
    if wpsq == Square::A7
        && wrsq == Square::A8
        && (bksq == Square::H7 || bksq == Square::G7)
        && brsq.file() == FILE_A
        && (brsq.rank() <= RANK_3 || wksq.file() >= FILE_D || wksq.rank() <= RANK_5)
    {
        return Some(0);
    }

    // If the defending king blocks the pawn and the attacking king is too far away, it is drawn
    if rank <= RANK_5
        && bksq == north(wpsq)
        && dist(wksq, wpsq) - tempo >= 2
        && dist(wksq, brsq) - tempo >= 2
    {
        return Some(0);
    }

    // Pawn on 7th rank supported behind by the rook usually wins if attacking king is closer to
    // the queening square than the defenders, and the defender cannot gain tempo by threatening
    // the rook attacking rook
    if rank == RANK_7
        && file != FILE_A
        && wrsq.file() == file
        && wrsq != queening_sq
        && dist(wksq, queening_sq) < dist(bksq, queening_sq) - 2 + tempo
        && dist(wksq, queening_sq) < dist(bksq, wrsq) + tempo
    {
        // return Some(SCALE_NORMAL - 2 * dist(wksq, queening_sq) as i32);
        return Some(SCALE_NORMAL);
    }

    // Similar to above but with pawn further back
    if file != FILE_A && wrsq.file() == file && wrsq.rank() < wpsq.rank() {
        let pawn_front = north(wpsq);

        if dist(wksq, queening_sq) < dist(bksq, queening_sq) - 2 + tempo
            && dist(wksq, pawn_front) < dist(bksq, pawn_front) - 2 + tempo
            && (dist(bksq, wrsq) + tempo >= 3
                || dist(wksq, queening_sq) < dist(bksq, wrsq) + tempo
                    && dist(wksq, pawn_front) < dist(bksq, wrsq) + tempo)
        {
            // return Some(
            //     SCALE_NORMAL
            //         - 8 * dist(wpsq, queening_sq) as i32
            //         - 2 * dist(wksq, queening_sq) as i32,
            // );
            return Some(SCALE_NORMAL);
        }
    }

    // If the pawn is not far advanced and the defending king is somewhere in the pawns path, it is
    // probably a draw
    if rank <= RANK_4 && bksq.rank() > wpsq.rank() {
        if bksq.file() == wpsq.file() {
            return Some(10);
        }

        if file_dist(bksq, wpsq) == 1 && dist(wksq, bksq) > 2 {
            return Some(24 - 2 * dist(wksq, bksq) as i32);
        }
    }

    None
}

fn wrong_rook_pawn_bishop_scale(pos: &Position, strong: Colour, info: &EvalInfo) -> Option<i32> {
    let weak = strong.opposite();

    if info.majors(Colour::Both) != 0
        || info.knights(Colour::Both) != 0
        || info.bishops(strong) != 1
        || info.bishops(weak) != 0
        || info.pawns(strong) == 0
        || info.pawns(weak) != 0
    {
        return None;
    }

    let pawns = pos.pieces[strong.idx()][Piece::Pawn.idx()];
    let only_a_pawns = (pawns & !Bitboard::FILE_A).is_empty();
    let only_h_pawns = (pawns & !Bitboard::FILE_H).is_empty();

    if !only_a_pawns && !only_h_pawns {
        return None;
    }

    let promotion_file = if only_a_pawns { FILE_A } else { FILE_H };
    let promotion_rank = if strong == Colour::White {
        RANK_8
    } else {
        RANK_1
    };
    let promotion_sq = Square::from_coords(promotion_rank, promotion_file);
    let bishop = pos.pieces[strong.idx()][Piece::Bishop.idx()].lsb();

    // Bishop guards the promotion square
    if bishop.colour() == promotion_sq.colour() {
        return None;
    }

    // Defending king can block the pawn from promoting
    if dist(pos.king_square(weak), promotion_sq) <= 1 {
        return Some(0);
    }

    None
}

fn rook_pawn_vs_bare_king_scale(pos: &Position, strong: Colour, info: &EvalInfo) -> Option<i32> {
    let weak = strong.opposite();

    if info.non_king(weak) != 0
        || info.majors(strong) != 0
        || info.minors(strong) != 0
        || info.pawns(strong) == 0
    {
        return None;
    }

    let pawns = pos.pieces[strong.idx()][Piece::Pawn.idx()];
    let only_a_pawns = (pawns & !Bitboard::FILE_A).is_empty();
    let only_h_pawns = (pawns & !Bitboard::FILE_H).is_empty();

    if !only_a_pawns || !only_h_pawns {
        return None;
    }

    let promotion_file = if only_a_pawns { FILE_A } else { FILE_H };
    let promotion_rank = if strong == Colour::White {
        RANK_8
    } else {
        RANK_1
    };
    let promotion_sq = Square::from_coords(promotion_rank, promotion_file);

    if pos.king_square(weak) == promotion_sq {
        return Some(0);
    }

    None
}

fn draw_scale(pos: &Position, strong: Colour, info: &EvalInfo, params: &Params) -> i32 {
    let weak = strong.opposite();
    let scales = params.draw_scales;

    if let Some(scale) = krpkr_scale(pos, info) {
        return scale;
    }

    if let Some(scale) = wrong_rook_pawn_bishop_scale(pos, strong, info) {
        return scale;
    }

    if let Some(scale) = rook_pawn_vs_bare_king_scale(pos, strong, info) {
        return scale;
    }

    // Scale down to near 0 when position is KNN vs K
    if info.non_king(weak) == 0 && info.non_king(strong) == 2 && info.knights(strong) == 2 {
        return scales.knn_vs_k;
    }

    // Scale down when the stronger side has no pawns
    if info.pawns(strong) == 0 {
        // No pawns makes it harder to convert, however a
        // queen is usually strong enough without
        if info.queens(strong) > 0 {
            return scales.no_pawn_queen;
        }

        // Rook advantages with no pawns can be hard to convert
        // unless the opponent has weak material
        if info.rooks(strong) > 0 {
            return if info.queens(weak) == 0 {
                scales.no_pawn_rook
            } else {
                scales.no_pawn_rook_vs_queen
            };
        }

        // KM vs K is a drawn
        if info.minors(Colour::Both) == 1 {
            return scales.no_pawn_minor;
        }
        // KNN vs K is almost always drawn, so scale to near 0
        if info.knights(strong) == 2 && info.minors(Colour::Both) == 2 {
            return scales.knn_vs_k;
        }
        // KMM vs K is usually winnable
        if info.minors(strong) == 2 {
            return SCALE_NORMAL;
        }
    }

    // Scale down opposite coloured bishops + pawn endgames
    if info.majors(Colour::Both) == 0
        && info.knights(Colour::Both) == 0
        && info.bishops(strong) == 1
        && info.bishops(weak) == 1
    {
        let strong_bishop = pos.pieces[strong.idx()][Piece::Bishop.idx()].lsb();
        let weak_bishop = pos.pieces[weak.idx()][Piece::Bishop.idx()].lsb();

        if strong_bishop.colour() == weak_bishop.colour() {
            return SCALE_NORMAL;
        }

        // Opposite coloured bishop vs opposite coloured bishop is nearly always drawn
        if info.pawns(Colour::Both) == 0 {
            return scales.no_pawn_minor;
        }

        let pawn_adv = info.pawns(strong) as i32 - info.pawns(weak) as i32;
        return scales.opposite_bishops[pawn_adv.clamp(0, 4) as usize];
    }

    // Scale down minor-only endings with < 3 pawns
    if info.majors(Colour::Both) == 0 {
        if info.minors(Colour::Both) == 0 || info.pawns(Colour::Both) >= 3 {
            return SCALE_NORMAL;
        }

        let minor_adv = info.minors(strong) as i32 - info.minors(weak) as i32;
        let pawn_adv = info.pawns(strong) as i32 - info.pawns(weak) as i32;
        return match minor_adv {
            // Eval likes strong side despite not being up a minor
            // With < 3 pawns, this is often hard to convert
            i32::MIN..=-1 => scales.minor_low_pawn[0],

            // Same minor count: mostly pawn conversion chances
            0 => match pawn_adv {
                i32::MIN..=0 => scales.minor_low_pawn[1],
                1 => scales.minor_low_pawn[2],
                _ => scales.minor_low_pawn[3],
            },

            // Up one minor in low-pawn endgame
            1 => match info.pawns(Colour::Both) {
                0 => scales.minor_low_pawn[4],
                1 => scales.minor_low_pawn[5],
                _ => scales.minor_low_pawn[6],
            },

            // Up two or more minors: likely win
            _ => scales.minor_low_pawn[7],
        };
    }

    // Scale down rook vs rook endgames
    if info.queens(Colour::Both) == 0
        && info.minors(Colour::Both) == 0
        && info.rooks(strong) == 1
        && info.rooks(weak) == 1
    {
        let total_pawns = info.pawns(Colour::Both);

        if total_pawns == 0 {
            return SCALE_NORMAL;
        }

        let pawn_adv = info.pawns(strong) as i32 - info.pawns(weak) as i32;
        return match pawn_adv {
            i32::MIN..=0 => scales.rook_vs_rook[0],
            1 => {
                if total_pawns <= 2 {
                    scales.rook_vs_rook[1]
                } else if total_pawns <= 4 {
                    scales.rook_vs_rook[2]
                } else {
                    scales.rook_vs_rook[3]
                }
            }
            2 => {
                if total_pawns <= 4 {
                    scales.rook_vs_rook[4]
                } else {
                    scales.rook_vs_rook[5]
                }
            }
            _ => scales.rook_vs_rook[6],
        };
    }

    SCALE_NORMAL
}

pub fn scale_endgame(pos: &Position, eval: Eval, info: &EvalInfo, params: &Params) -> Eval {
    if eval == 0 {
        return 0;
    }

    let strong = if eval > 0 {
        pos.side_to_move
    } else {
        pos.side_to_move.opposite()
    };

    let scale = draw_scale(pos, strong, info, params);
    eval * scale / SCALE_NORMAL
}
