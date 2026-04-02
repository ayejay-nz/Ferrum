use crate::{
    position::Position,
    types::{Colour, Piece, Square},
};

pub type Eval = i32;
pub const INFINITY: Eval = 32001;

const PIECE_VALUES: [Score; 6] = [
    Score { mg: 82, eg: 94 },    // Pawn
    Score { mg: 337, eg: 281 },  // Knight
    Score { mg: 365, eg: 297 },  // Bishop
    Score { mg: 477, eg: 512 },  // Rook
    Score { mg: 1025, eg: 936 }, // Queen
    Score { mg: 0, eg: 0 },      // King
];

type PST = [Score; 64];

#[rustfmt::skip]
const PAWN_PST: PST = [
    Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 },
    Score { mg:   98, eg:  178 }, Score { mg:  134, eg:  173 }, Score { mg:   61, eg:  158 }, Score { mg:   95, eg:  134 }, Score { mg:   68, eg:  147 }, Score { mg:  126, eg:  132 }, Score { mg:   34, eg:  165 }, Score { mg:  -11, eg:  187 },
    Score { mg:   -6, eg:   94 }, Score { mg:    7, eg:  100 }, Score { mg:   26, eg:   85 }, Score { mg:   31, eg:   67 }, Score { mg:   65, eg:   56 }, Score { mg:   56, eg:   53 }, Score { mg:   25, eg:   82 }, Score { mg:  -20, eg:   84 },
    Score { mg:  -14, eg:   32 }, Score { mg:   13, eg:   24 }, Score { mg:    6, eg:   13 }, Score { mg:   21, eg:    5 }, Score { mg:   23, eg:   -2 }, Score { mg:   12, eg:    4 }, Score { mg:   17, eg:   17 }, Score { mg:  -23, eg:   17 },
    Score { mg:  -27, eg:   13 }, Score { mg:   -2, eg:    9 }, Score { mg:   -5, eg:   -3 }, Score { mg:   12, eg:   -7 }, Score { mg:   17, eg:   -7 }, Score { mg:    6, eg:   -8 }, Score { mg:   10, eg:    3 }, Score { mg:  -25, eg:   -1 },
    Score { mg:  -26, eg:    4 }, Score { mg:   -4, eg:    7 }, Score { mg:   -4, eg:   -6 }, Score { mg:  -10, eg:    1 }, Score { mg:    3, eg:    0 }, Score { mg:    3, eg:   -5 }, Score { mg:   33, eg:   -1 }, Score { mg:  -12, eg:   -8 },
    Score { mg:  -35, eg:   13 }, Score { mg:   -1, eg:    8 }, Score { mg:  -20, eg:    8 }, Score { mg:  -23, eg:   10 }, Score { mg:  -15, eg:   13 }, Score { mg:   24, eg:    0 }, Score { mg:   38, eg:    2 }, Score { mg:  -22, eg:   -7 },
    Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }, Score { mg:    0, eg:    0 }
];

#[rustfmt::skip]
const KNIGHT_PST: PST = [
    Score { mg: -167, eg:  -58 }, Score { mg:  -89, eg:  -38 }, Score { mg:  -34, eg:  -13 }, Score { mg:  -49, eg:  -28 }, Score { mg:   61, eg:  -31 }, Score { mg:  -97, eg:  -27 }, Score { mg:  -15, eg:  -63 }, Score { mg: -107, eg:  -99 },
    Score { mg:  -73, eg:  -25 }, Score { mg:  -41, eg:   -8 }, Score { mg:   72, eg:  -25 }, Score { mg:   36, eg:   -2 }, Score { mg:   23, eg:   -9 }, Score { mg:   62, eg:  -25 }, Score { mg:    7, eg:  -24 }, Score { mg:  -17, eg:  -52 },
    Score { mg:  -47, eg:  -24 }, Score { mg:   60, eg:  -20 }, Score { mg:   37, eg:   10 }, Score { mg:   65, eg:    9 }, Score { mg:   84, eg:   -1 }, Score { mg:  129, eg:   -9 }, Score { mg:   73, eg:  -19 }, Score { mg:   44, eg:  -41 },
    Score { mg:   -9, eg:  -17 }, Score { mg:   17, eg:    3 }, Score { mg:   19, eg:   22 }, Score { mg:   53, eg:   22 }, Score { mg:   37, eg:   22 }, Score { mg:   69, eg:   11 }, Score { mg:   18, eg:    8 }, Score { mg:   22, eg:  -18 },
    Score { mg:  -13, eg:  -18 }, Score { mg:    4, eg:   -6 }, Score { mg:   16, eg:   16 }, Score { mg:   13, eg:   25 }, Score { mg:   28, eg:   16 }, Score { mg:   19, eg:   17 }, Score { mg:   21, eg:    4 }, Score { mg:   -8, eg:  -18 },
    Score { mg:  -23, eg:  -23 }, Score { mg:   -9, eg:   -3 }, Score { mg:   12, eg:   -1 }, Score { mg:   10, eg:   15 }, Score { mg:   19, eg:   10 }, Score { mg:   17, eg:   -3 }, Score { mg:   25, eg:  -20 }, Score { mg:  -16, eg:  -22 },
    Score { mg:  -29, eg:  -42 }, Score { mg:  -53, eg:  -20 }, Score { mg:  -12, eg:  -10 }, Score { mg:   -3, eg:   -5 }, Score { mg:   -1, eg:   -2 }, Score { mg:   18, eg:  -20 }, Score { mg:  -14, eg:  -23 }, Score { mg:  -19, eg:  -44 },
    Score { mg: -105, eg:  -29 }, Score { mg:  -21, eg:  -51 }, Score { mg:  -58, eg:  -23 }, Score { mg:  -33, eg:  -15 }, Score { mg:  -17, eg:  -22 }, Score { mg:  -28, eg:  -18 }, Score { mg:  -19, eg:  -50 }, Score { mg:  -23, eg:  -64 }
];

#[rustfmt::skip]
const BISHOP_PST: PST = [
    Score { mg:  -29, eg:  -14 }, Score { mg:    4, eg:  -21 }, Score { mg:  -82, eg:  -11 }, Score { mg:  -37, eg:   -8 }, Score { mg:  -25, eg:   -7 }, Score { mg:  -42, eg:   -9 }, Score { mg:    7, eg:  -17 }, Score { mg:   -8, eg:  -24 },
    Score { mg:  -26, eg:   -8 }, Score { mg:   16, eg:   -4 }, Score { mg:  -18, eg:    7 }, Score { mg:  -13, eg:  -12 }, Score { mg:   30, eg:   -3 }, Score { mg:   59, eg:  -13 }, Score { mg:   18, eg:   -4 }, Score { mg:  -47, eg:  -14 },
    Score { mg:  -16, eg:    2 }, Score { mg:   37, eg:   -8 }, Score { mg:   43, eg:    0 }, Score { mg:   40, eg:   -1 }, Score { mg:   35, eg:   -2 }, Score { mg:   50, eg:    6 }, Score { mg:   37, eg:    0 }, Score { mg:   -2, eg:    4 },
    Score { mg:   -4, eg:   -3 }, Score { mg:    5, eg:    9 }, Score { mg:   19, eg:   12 }, Score { mg:   50, eg:    9 }, Score { mg:   37, eg:   14 }, Score { mg:   37, eg:   10 }, Score { mg:    7, eg:    3 }, Score { mg:   -2, eg:    2 },
    Score { mg:   -6, eg:   -6 }, Score { mg:   13, eg:    3 }, Score { mg:   13, eg:   13 }, Score { mg:   26, eg:   19 }, Score { mg:   34, eg:    7 }, Score { mg:   12, eg:   10 }, Score { mg:   10, eg:   -3 }, Score { mg:    4, eg:   -9 },
    Score { mg:    0, eg:  -12 }, Score { mg:   15, eg:   -3 }, Score { mg:   15, eg:    8 }, Score { mg:   15, eg:   10 }, Score { mg:   14, eg:   13 }, Score { mg:   27, eg:    3 }, Score { mg:   18, eg:   -7 }, Score { mg:   10, eg:  -15 },
    Score { mg:    4, eg:  -14 }, Score { mg:   15, eg:  -18 }, Score { mg:   16, eg:   -7 }, Score { mg:    0, eg:   -1 }, Score { mg:    7, eg:    4 }, Score { mg:   21, eg:   -9 }, Score { mg:   33, eg:  -15 }, Score { mg:    1, eg:  -27 },
    Score { mg:  -33, eg:  -23 }, Score { mg:   -3, eg:   -9 }, Score { mg:  -14, eg:  -23 }, Score { mg:  -21, eg:   -5 }, Score { mg:  -13, eg:   -9 }, Score { mg:  -12, eg:  -16 }, Score { mg:  -39, eg:   -5 }, Score { mg:  -21, eg:  -17 }
];

#[rustfmt::skip]
const ROOK_PST: PST = [
    Score { mg:   32, eg:   13 }, Score { mg:   42, eg:   10 }, Score { mg:   32, eg:   18 }, Score { mg:   51, eg:   15 }, Score { mg:   63, eg:   12 }, Score { mg:    9, eg:   12 }, Score { mg:   31, eg:    8 }, Score { mg:   43, eg:    5 },
    Score { mg:   27, eg:   11 }, Score { mg:   32, eg:   13 }, Score { mg:   58, eg:   13 }, Score { mg:   62, eg:   11 }, Score { mg:   80, eg:   -3 }, Score { mg:   67, eg:    3 }, Score { mg:   26, eg:    8 }, Score { mg:   44, eg:    3 },
    Score { mg:   -5, eg:    7 }, Score { mg:   19, eg:    7 }, Score { mg:   26, eg:    7 }, Score { mg:   36, eg:    5 }, Score { mg:   17, eg:    4 }, Score { mg:   45, eg:   -3 }, Score { mg:   61, eg:   -5 }, Score { mg:   16, eg:   -3 },
    Score { mg:  -24, eg:    4 }, Score { mg:  -11, eg:    3 }, Score { mg:    7, eg:   13 }, Score { mg:   26, eg:    1 }, Score { mg:   24, eg:    2 }, Score { mg:   35, eg:    1 }, Score { mg:   -8, eg:   -1 }, Score { mg:  -20, eg:    2 },
    Score { mg:  -36, eg:    3 }, Score { mg:  -26, eg:    5 }, Score { mg:  -12, eg:    8 }, Score { mg:   -1, eg:    4 }, Score { mg:    9, eg:   -5 }, Score { mg:   -7, eg:   -6 }, Score { mg:    6, eg:   -8 }, Score { mg:  -23, eg:  -11 },
    Score { mg:  -45, eg:   -4 }, Score { mg:  -25, eg:    0 }, Score { mg:  -16, eg:   -5 }, Score { mg:  -17, eg:   -1 }, Score { mg:    3, eg:   -7 }, Score { mg:    0, eg:  -12 }, Score { mg:   -5, eg:   -8 }, Score { mg:  -33, eg:  -16 },
    Score { mg:  -44, eg:   -6 }, Score { mg:  -16, eg:   -6 }, Score { mg:  -20, eg:    0 }, Score { mg:   -9, eg:    2 }, Score { mg:   -1, eg:   -9 }, Score { mg:   11, eg:   -9 }, Score { mg:   -6, eg:  -11 }, Score { mg:  -71, eg:   -3 },
    Score { mg:  -19, eg:   -9 }, Score { mg:  -13, eg:    2 }, Score { mg:    1, eg:    3 }, Score { mg:   17, eg:   -1 }, Score { mg:   16, eg:   -5 }, Score { mg:    7, eg:  -13 }, Score { mg:  -37, eg:    4 }, Score { mg:  -26, eg:  -20 }
];

#[rustfmt::skip]
const QUEEN_PST: PST = [
    Score { mg:  -28, eg:   -9 }, Score { mg:    0, eg:   22 }, Score { mg:   29, eg:   22 }, Score { mg:   12, eg:   27 }, Score { mg:   59, eg:   27 }, Score { mg:   44, eg:   19 }, Score { mg:   43, eg:   10 }, Score { mg:   45, eg:   20 },
    Score { mg:  -24, eg:  -17 }, Score { mg:  -39, eg:   20 }, Score { mg:   -5, eg:   32 }, Score { mg:    1, eg:   41 }, Score { mg:  -16, eg:   58 }, Score { mg:   57, eg:   25 }, Score { mg:   28, eg:   30 }, Score { mg:   54, eg:    0 },
    Score { mg:  -13, eg:  -20 }, Score { mg:  -17, eg:    6 }, Score { mg:    7, eg:    9 }, Score { mg:    8, eg:   49 }, Score { mg:   29, eg:   47 }, Score { mg:   56, eg:   35 }, Score { mg:   47, eg:   19 }, Score { mg:   57, eg:    9 },
    Score { mg:  -27, eg:    3 }, Score { mg:  -27, eg:   22 }, Score { mg:  -16, eg:   24 }, Score { mg:  -16, eg:   45 }, Score { mg:   -1, eg:   57 }, Score { mg:   17, eg:   40 }, Score { mg:   -2, eg:   57 }, Score { mg:    1, eg:   36 },
    Score { mg:   -9, eg:  -18 }, Score { mg:  -26, eg:   28 }, Score { mg:   -9, eg:   19 }, Score { mg:  -10, eg:   47 }, Score { mg:   -2, eg:   31 }, Score { mg:   -4, eg:   34 }, Score { mg:    3, eg:   39 }, Score { mg:   -3, eg:   23 },
    Score { mg:  -14, eg:  -16 }, Score { mg:    2, eg:  -27 }, Score { mg:  -11, eg:   15 }, Score { mg:   -2, eg:    6 }, Score { mg:   -5, eg:    9 }, Score { mg:    2, eg:   17 }, Score { mg:   14, eg:   10 }, Score { mg:    5, eg:    5 },
    Score { mg:  -35, eg:  -22 }, Score { mg:   -8, eg:  -23 }, Score { mg:   11, eg:  -30 }, Score { mg:    2, eg:  -16 }, Score { mg:    8, eg:  -16 }, Score { mg:   15, eg:  -23 }, Score { mg:   -3, eg:  -36 }, Score { mg:    1, eg:  -32 },
    Score { mg:   -1, eg:  -33 }, Score { mg:  -18, eg:  -28 }, Score { mg:   -9, eg:  -22 }, Score { mg:   10, eg:  -43 }, Score { mg:  -15, eg:   -5 }, Score { mg:  -25, eg:  -32 }, Score { mg:  -31, eg:  -20 }, Score { mg:  -50, eg:  -41 }
];

#[rustfmt::skip]
const KING_PST: PST = [
    Score { mg:  -65, eg:  -74 }, Score { mg:   23, eg:  -35 }, Score { mg:   16, eg:  -18 }, Score { mg:  -15, eg:  -18 }, Score { mg:  -56, eg:  -11 }, Score { mg:  -34, eg:   15 }, Score { mg:    2, eg:    4 }, Score { mg:   13, eg:  -17 },
    Score { mg:   29, eg:  -12 }, Score { mg:   -1, eg:   17 }, Score { mg:  -20, eg:   14 }, Score { mg:   -7, eg:   17 }, Score { mg:   -8, eg:   17 }, Score { mg:   -4, eg:   38 }, Score { mg:  -38, eg:   23 }, Score { mg:  -29, eg:   11 },
    Score { mg:   -9, eg:   10 }, Score { mg:   24, eg:   17 }, Score { mg:    2, eg:   23 }, Score { mg:  -16, eg:   15 }, Score { mg:  -20, eg:   20 }, Score { mg:    6, eg:   45 }, Score { mg:   22, eg:   44 }, Score { mg:  -22, eg:   13 },
    Score { mg:  -17, eg:   -8 }, Score { mg:  -20, eg:   22 }, Score { mg:  -12, eg:   24 }, Score { mg:  -27, eg:   27 }, Score { mg:  -30, eg:   26 }, Score { mg:  -25, eg:   33 }, Score { mg:  -14, eg:   26 }, Score { mg:  -36, eg:    3 },
    Score { mg:  -49, eg:  -18 }, Score { mg:   -1, eg:   -4 }, Score { mg:  -27, eg:   21 }, Score { mg:  -39, eg:   24 }, Score { mg:  -46, eg:   27 }, Score { mg:  -44, eg:   23 }, Score { mg:  -33, eg:    9 }, Score { mg:  -51, eg:  -11 },
    Score { mg:  -14, eg:  -19 }, Score { mg:  -14, eg:   -3 }, Score { mg:  -22, eg:   11 }, Score { mg:  -46, eg:   21 }, Score { mg:  -44, eg:   23 }, Score { mg:  -30, eg:   16 }, Score { mg:  -15, eg:    7 }, Score { mg:  -27, eg:   -9 },
    Score { mg:    1, eg:  -27 }, Score { mg:    7, eg:  -11 }, Score { mg:   -8, eg:    4 }, Score { mg:  -64, eg:   13 }, Score { mg:  -43, eg:   14 }, Score { mg:  -16, eg:    4 }, Score { mg:    9, eg:   -5 }, Score { mg:    8, eg:  -17 },
    Score { mg:  -15, eg:  -53 }, Score { mg:   36, eg:  -34 }, Score { mg:   12, eg:  -21 }, Score { mg:  -54, eg:  -11 }, Score { mg:    8, eg:  -28 }, Score { mg:  -28, eg:  -14 }, Score { mg:   24, eg:  -24 }, Score { mg:   14, eg:  -43 }
];

const PSTS: [PST; 6] = [
    PAWN_PST,
    KNIGHT_PST,
    BISHOP_PST,
    ROOK_PST,
    QUEEN_PST,
    KING_PST,
];

const PHASE_WEIGHTS: [i32; 6] = [0, 1, 1, 2, 4, 0];

#[derive(Copy, Clone, Default)]
struct Score {
    mg: Eval,
    eg: Eval,
}

impl Score {
    fn add(&mut self, sign: Eval, mg: Eval, eg: Eval) {
        self.mg += sign * mg;
        self.eg += sign * eg;
    }
}

#[inline(always)]
const fn piece_values(piece: Piece) -> Score {
    PIECE_VALUES[piece.idx()]
}

#[inline(always)]
const fn phase_weight(piece: Piece) -> i32 {
    PHASE_WEIGHTS[piece.idx()]
}

#[inline(always)]
const fn relative_square(colour: Colour, sq: Square) -> usize {
    match colour {
        Colour::Black => sq.idx(),
        Colour::White => sq.idx() ^ 56,
    }
}

fn eval_material_pst(pos: &Position, score: &mut Score) -> i32 {
    let mut phase = 0;

    for colour in [Colour::White, Colour::Black] {
        let sign = if colour == Colour::White { 1 } else { -1 };

        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            let values = piece_values(piece);
            let mut bb = pos.pieces[colour.idx()][piece.idx()];

            while !bb.is_empty() {
                let lsb = bb.pop_lsb();
                let sq = relative_square(colour, lsb);

                score.mg += sign * (values.mg + PSTS[piece.idx()][sq].mg);
                score.eg += sign * (values.eg + PSTS[piece.idx()][sq].eg);

                phase += phase_weight(piece);
            }
        }
    }

    phase
}

fn taper(score: Score, phase: i32, us: Colour) -> Eval {
    let mg_phase = phase.min(24);
    let eg_phase = 24 - mg_phase;

    let score = (mg_phase * score.mg + eg_phase * score.eg) / 24;

    if us == Colour::White { score } else { -score }
}

pub fn evaluate(pos: &Position) -> Eval {
    let mut score = Score::default();

    let phase = eval_material_pst(pos, &mut score);

    taper(score, phase, pos.side_to_move)
}
