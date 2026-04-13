use crate::{
    evaluate::Score,
    tune::{LazyParams, Params, TunableParams},
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

fn fmt_pst(name: &str, pst: &[Score; 64]) -> String {
    let mut out = String::new();
    out.push_str("#[rustfmt::skip]\n");
    out.push_str(&format!("pub const {name}: [Score; 64] = [\n"));

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

pub fn dump_full_params(label: &str, theta: &[i32], loss: f64) {
    let params = Params::unpack(theta);

    println!("==================================================");
    println!("{label}_loss = {loss}");
    println!("{label}_theta = {:?}", theta);
    println!("--------------------------------------------------");

    println!(
        "pub const PIECE_VALUES: [Score; 5] = {};",
        fmt_score_array(&[
            params.pawn_value,
            params.knight_value,
            params.bishop_value,
            params.rook_value,
            params.queen_value
        ]),
    );
    println!();

    println!(
        "pub const DOUBLED_PAWNS: Score = {};",
        fmt_score(params.doubled_pawns)
    );
    println!(
        "pub const TRIPLED_PAWNS: Score = {};",
        fmt_score(params.tripled_pawns)
    );
    println!(
        "pub const QUADRUPLED_PAWNS: Score = {};",
        fmt_score(params.quadrupled_pawns)
    );
    println!();
    println!(
        "pub const ISOLATED_PAWN: [Score; 4] = {};",
        fmt_score_array(&params.isolated_pawn)
    );
    println!(
        "pub const PASSED_PAWN: [Score; 6] = {};",
        fmt_score_array(&params.passed_pawn)
    );
    println!();

    println!(
        "pub const KNIGHT_OUTPOST: Score = {};",
        fmt_score(params.knight_outpost)
    );
    println!(
        "pub const DEFENDED_KNIGHT_OUTPOST: Score = {};",
        fmt_score(params.defended_knight_outpost)
    );
    println!();

    println!(
        "pub const BISHOP_PAIR: Score = {};",
        fmt_score(params.bishop_pair)
    );
    println!(
        "pub const BISHOP_SAME_COLOUR_PAWNS: [Score; 9] = {};",
        fmt_score_array(&params.bishop_same_colour_pawns)
    );
    println!();

    println!(
        "pub const ROOK_OPEN_FILE: Score = {};",
        fmt_score(params.rook_open_file)
    );
    println!(
        "pub const ROOK_SEMI_OPEN_FILE: Score = {};",
        fmt_score(params.rook_semi_open_file)
    );
    println!();

    println!(
        "pub const KING_ON_SEMI_OPEN_FILE: Score = {};",
        fmt_score(params.king_on_semi_open_file)
    );
    println!(
        "pub const KING_ON_OPEN_FILE: Score = {};",
        fmt_score(params.king_on_open_file)
    );
    println!(
        "pub const KING_PAWN_SHIELD_DISTANCE: [Score; 4] = {};",
        fmt_score_array(&params.king_pawn_shield_distance)
    );
    println!(
        "pub const KING_SHIELD_MISSING_PAWN: Score = {};",
        fmt_score(params.king_shield_missing_pawn)
    );
    println!(
        "pub const ENEMY_PAWN_DISTANCE_FROM_BACKRANK: [Score; 4] = {};",
        fmt_score_array(&params.enemy_pawn_distance_from_backrank)
    );
    println!();

    println!(
        "pub const KING_RING_PAWN_WEIGHT: Score = {};",
        fmt_score(params.king_ring_pawn_weight)
    );
    println!(
        "pub const KING_RING_KNIGHT_WEIGHT: Score = {};",
        fmt_score(params.king_ring_knight_weight)
    );
    println!(
        "pub const KING_RING_BISHOP_WEIGHT: Score = {};",
        fmt_score(params.king_ring_bishop_weight)
    );
    println!(
        "pub const KING_RING_ROOK_WEIGHT: Score = {};",
        fmt_score(params.king_ring_rook_weight)
    );
    println!(
        "pub const KING_RING_QUEEN_WEIGHT: Score = {};",
        fmt_score(params.king_ring_queen_weight)
    );
    println!(
        "pub const KING_RING_ATTACKS: [Score; 24] = {};",
        fmt_score_array(&params.king_ring_attacks)
    );
    println!();

    println!("// Adjustment values based on the number of pawns left");
    println!(
        "pub const KNIGHT_ADJ: [Score; 9] = {};",
        fmt_score_array(&params.knight_adj)
    );
    println!(
        "pub const ROOK_ADJ: [Score; 9] = {};",
        fmt_score_array(&params.rook_adj)
    );
    println!();

    println!("// Mobility scores based on the number of moves available to a piece");
    println!(
        "pub const KNIGHT_MOBILITY: [Score; 9] = {};",
        fmt_score_array(&params.knight_mobility)
    );
    println!(
        "pub const BISHOP_MOBILITY: [Score; 14] = {};",
        fmt_score_array(&params.bishop_mobility)
    );
    println!(
        "pub const ROOK_MOBILITY: [Score; 15] = {};",
        fmt_score_array(&params.rook_mobility)
    );
    println!(
        "pub const QUEEN_MOBILITY: [Score; 28] = {};",
        fmt_score_array(&params.queen_mobility)
    );
    println!();

    println!("// Piece square tables");
    println!("{}", fmt_pst("PAWN_PST", &params.pawn_pst));
    println!("{}", fmt_pst("KNIGHT_PST", &params.knight_pst));
    println!("{}", fmt_pst("BISHOP_PST", &params.bishop_pst));
    println!("{}", fmt_pst("ROOK_PST", &params.rook_pst));
    println!("{}", fmt_pst("QUEEN_PST", &params.queen_pst));
    println!("{}", fmt_pst("KING_PST", &params.king_pst));
}

pub fn dump_lazy_params(label: &str, theta: &[i32], loss: f64) {
    let params = LazyParams::unpack(theta);

    println!("==================================================");
    println!("{label}_loss = {loss}");
    println!("{label}_theta = {:?}", theta);
    println!("--------------------------------------------------");

    println!(
        "pub const LAZY_PIECE_VALUES: [Score; 5] = {};",
        fmt_score_array(&[
            params.pawn_value,
            params.knight_value,
            params.bishop_value,
            params.rook_value,
            params.queen_value
        ]),
    );
    println!();

    println!("// Lazy PSTs");
    println!("{}", fmt_pst("LAZY_PAWN_PST", &params.pawn_pst));
    println!("{}", fmt_pst("LAZY_KNIGHT_PST", &params.knight_pst));
    println!("{}", fmt_pst("LAZY_BISHOP_PST", &params.bishop_pst));
    println!("{}", fmt_pst("LAZY_ROOK_PST", &params.rook_pst));
    println!("{}", fmt_pst("LAZY_QUEEN_PST", &params.queen_pst));
    println!("{}", fmt_pst("LAZY_KING_PST", &params.king_pst));
}
