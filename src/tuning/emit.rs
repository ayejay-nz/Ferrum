use std::fmt::Write as _;

use crate::{
    evaluate::Score,
    params::PST,
    tuning::types::{FullTuningConfig, LazyTuningConfig, TuningConfig},
};

pub fn fmt_score(s: Score) -> String {
    format!("s!({}, {})", s.mg, s.eg)
}

fn fmt_pst_score(s: Score) -> String {
    format!("s!({:>4}, {:>4})", s.mg, s.eg)
}

fn fmt_score_array<const N: usize>(arr: &[Score; N]) -> String {
    let items = arr
        .iter()
        .map(|&s| fmt_score(s))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{items}]")
}

fn fmt_pst(name: &str, pst: &PST) -> String {
    let mut out = String::new();
    out.push_str("#[rustfmt::skip]\n");
    out.push_str(&format!("pub const {name}: PST = [\n"));

    for row in 0..8 {
        out.push_str("    ");

        for col in 0..8 {
            let idx = row * 8 + col;
            out.push_str(&fmt_pst_score(pst[idx]));

            if idx != 63 {
                out.push_str(", ");
            }
        }

        out.push('\n');
    }

    out.push_str("];");
    out
}

pub fn render_full_params(label: &str, theta: &[i32], loss: f64) -> String {
    let params = FullTuningConfig::unpack(theta);
    let mut out = String::new();

    writeln!(
        &mut out,
        "=================================================="
    )
    .unwrap();
    writeln!(&mut out, "{label}_loss = {loss}").unwrap();
    writeln!(&mut out, "{label}_theta = {:?}", theta).unwrap();
    writeln!(
        &mut out,
        "--------------------------------------------------"
    )
    .unwrap();

    writeln!(
        &mut out,
        "pub const PIECE_VALUES: [Score; 5] = {};",
        fmt_score_array(&[
            params.pawn_value,
            params.knight_value,
            params.bishop_value,
            params.rook_value,
            params.queen_value
        ]),
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "pub const DOUBLED_PAWNS: Score = {};",
        fmt_score(params.doubled_pawns)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const TRIPLED_PAWNS: Score = {};",
        fmt_score(params.tripled_pawns)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const QUADRUPLED_PAWNS: Score = {};",
        fmt_score(params.quadrupled_pawns)
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "pub const ISOLATED_PAWN: [Score; 4] = {};",
        fmt_score_array(&params.isolated_pawn)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const BACKWARD_PAWN: [Score; 4] = {};",
        fmt_score_array(&params.backward_pawn)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const WEAK_UNOPPOSED: Score = {};",
        fmt_score(params.weak_unopposed)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const CANDIDATE_PASSER: [Score; 6] = {};",
        fmt_score_array(&params.candidate_passer)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const CONNECTED_BONUS: [Score; 6] = {};",
        fmt_score_array(&params.connected_bonus)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const SUPPORTED_BONUS: [Score; 3] = {};",
        fmt_score_array(&params.supported_bonus)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const PASSED_PAWN: [Score; 6] = {};",
        fmt_score_array(&params.passed_pawn)
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "pub const KNIGHT_OUTPOST: Score = {};",
        fmt_score(params.knight_outpost)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const DEFENDED_KNIGHT_OUTPOST: Score = {};",
        fmt_score(params.defended_knight_outpost)
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "pub const BISHOP_PAIR: Score = {};",
        fmt_score(params.bishop_pair)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const BISHOP_SAME_COLOUR_PAWNS: [Score; 9] = {};",
        fmt_score_array(&params.bishop_same_colour_pawns)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const FIANCHETTO: Score = {};",
        fmt_score(params.fianchetto)
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "pub const ROOK_OPEN_FILE: Score = {};",
        fmt_score(params.rook_open_file)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const ROOK_SEMI_OPEN_FILE: Score = {};",
        fmt_score(params.rook_semi_open_file)
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "pub const QUEEN_UNDEVELOPED_PIECE_PUNISHMENT: Score = {};",
        fmt_score(params.queen_undeveloped_piece_punishment)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const QUEEN_UNMOVED_KING_PUNISHMENT: Score = {};",
        fmt_score(params.queen_unmoved_king_punishment)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const PAWN_THREAT_MINOR: Score = {};",
        fmt_score(params.pawn_threat_minor)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const PAWN_THREAT_MAJOR: Score = {};",
        fmt_score(params.pawn_threat_major)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const HANGING_MINOR: Score = {};",
        fmt_score(params.hanging_minor)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const HANGING_ROOK: Score = {};",
        fmt_score(params.hanging_rook)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const HANGING_QUEEN: Score = {};",
        fmt_score(params.hanging_queen)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const MINOR_THREAT_QUEEN: Score = {};",
        fmt_score(params.minor_threat_queen)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const ROOK_THREAT_QUEEN: Score = {};",
        fmt_score(params.rook_threat_queen)
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "pub const KING_ON_SEMI_OPEN_FILE: Score = {};",
        fmt_score(params.king_on_semi_open_file)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KING_ON_OPEN_FILE: Score = {};",
        fmt_score(params.king_on_open_file)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KING_PAWN_SHIELD_DISTANCE: [Score; 4] = {};",
        fmt_score_array(&params.king_pawn_shield_distance)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KING_SHIELD_MISSING_PAWN: Score = {};",
        fmt_score(params.king_shield_missing_pawn)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const ENEMY_PAWN_DISTANCE_FROM_BACKRANK: [Score; 4] = {};",
        fmt_score_array(&params.enemy_pawn_distance_from_backrank)
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "pub const KING_RING_PAWN_WEIGHT: Score = {};",
        fmt_score(params.king_ring_pawn_weight)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KING_RING_KNIGHT_WEIGHT: Score = {};",
        fmt_score(params.king_ring_knight_weight)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KING_RING_BISHOP_WEIGHT: Score = {};",
        fmt_score(params.king_ring_bishop_weight)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KING_RING_ROOK_WEIGHT: Score = {};",
        fmt_score(params.king_ring_rook_weight)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KING_RING_QUEEN_WEIGHT: Score = {};",
        fmt_score(params.king_ring_queen_weight)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KING_RING_ATTACKS: [Score; 24] = {};",
        fmt_score_array(&params.king_ring_attacks)
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "// Adjustment values based on the number of pawns left"
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KNIGHT_ADJ: [Score; 9] = {};",
        fmt_score_array(&params.knight_adj)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const ROOK_ADJ: [Score; 9] = {};",
        fmt_score_array(&params.rook_adj)
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(
        &mut out,
        "// Mobility scores based on the number of moves available to a piece"
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const KNIGHT_MOBILITY: [Score; 9] = {};",
        fmt_score_array(&params.knight_mobility)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const BISHOP_MOBILITY: [Score; 14] = {};",
        fmt_score_array(&params.bishop_mobility)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const ROOK_MOBILITY: [Score; 15] = {};",
        fmt_score_array(&params.rook_mobility)
    )
    .unwrap();
    writeln!(
        &mut out,
        "pub const QUEEN_MOBILITY: [Score; 28] = {};",
        fmt_score_array(&params.queen_mobility)
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "// Piece square tables").unwrap();
    writeln!(&mut out, "{}", fmt_pst("PAWN_PST", &params.pawn_pst)).unwrap();
    writeln!(&mut out, "{}", fmt_pst("KNIGHT_PST", &params.knight_pst)).unwrap();
    writeln!(&mut out, "{}", fmt_pst("BISHOP_PST", &params.bishop_pst)).unwrap();
    writeln!(&mut out, "{}", fmt_pst("ROOK_PST", &params.rook_pst)).unwrap();
    writeln!(&mut out, "{}", fmt_pst("QUEEN_PST", &params.queen_pst)).unwrap();
    writeln!(&mut out, "{}", fmt_pst("KING_PST", &params.king_pst)).unwrap();

    out
}

pub fn dump_full_params(label: &str, theta: &[i32], loss: f64) {
    print!("{}", render_full_params(label, theta, loss));
}

pub fn render_lazy_params(label: &str, theta: &[i32], loss: f64) -> String {
    let params = LazyTuningConfig::unpack(theta);
    let mut out = String::new();

    writeln!(
        &mut out,
        "=================================================="
    )
    .unwrap();
    writeln!(&mut out, "{label}_loss = {loss}").unwrap();
    writeln!(&mut out, "{label}_theta = {:?}", theta).unwrap();
    writeln!(
        &mut out,
        "--------------------------------------------------"
    )
    .unwrap();

    writeln!(
        &mut out,
        "pub const LAZY_PIECE_VALUES: [Score; 5] = {};",
        fmt_score_array(&[
            params.pawn_value,
            params.knight_value,
            params.bishop_value,
            params.rook_value,
            params.queen_value
        ]),
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "// Lazy PSTs").unwrap();
    writeln!(&mut out, "{}", fmt_pst("LAZY_PAWN_PST", &params.pawn_pst)).unwrap();
    writeln!(
        &mut out,
        "{}",
        fmt_pst("LAZY_KNIGHT_PST", &params.knight_pst)
    )
    .unwrap();
    writeln!(
        &mut out,
        "{}",
        fmt_pst("LAZY_BISHOP_PST", &params.bishop_pst)
    )
    .unwrap();
    writeln!(&mut out, "{}", fmt_pst("LAZY_ROOK_PST", &params.rook_pst)).unwrap();
    writeln!(&mut out, "{}", fmt_pst("LAZY_QUEEN_PST", &params.queen_pst)).unwrap();
    writeln!(&mut out, "{}", fmt_pst("LAZY_KING_PST", &params.king_pst)).unwrap();

    out
}

pub fn dump_lazy_params(label: &str, theta: &[i32], loss: f64) {
    print!("{}", render_lazy_params(label, theta, loss));
}
