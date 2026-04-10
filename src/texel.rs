use core::f64;
use rayon::prelude::*;
use std::{
    cmp::Reverse,
    fmt,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    evaluate::{Eval, INFINITY, evaluate_with},
    movegen::generate_legal_noisy,
    position::{Position, StateInfo},
    search::OrderingTables,
    tune::{DEFAULT_PARAMS, ParamBounds, Params},
    types::Colour,
};
use crate::{
    movegen::{MoveList, generate_legal},
    types::{CastlingType, Move, Piece, Square},
};
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum GameResult {
    BlackWin,
    Draw,
    WhiteWin,
}

impl GameResult {
    pub fn from_pgn_tag(tag: &str) -> Option<Self> {
        match tag {
            "0-1" => Some(Self::BlackWin),
            "1-0" => Some(Self::WhiteWin),
            "1/2-1/2" => Some(Self::Draw),
            "*" => None,
            _ => None,
        }
    }

    pub fn to_value(self) -> f64 {
        match self {
            Self::BlackWin => 0f64,
            Self::WhiteWin => 1f64,
            Self::Draw => 0.5,
        }
    }
}

impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlackWin => f.write_str("0-1"),
            Self::WhiteWin => f.write_str("1-0"),
            Self::Draw => f.write_str("1/2-1/2"),
        }
    }
}

fn split_games(text: &str) -> Vec<&str> {
    let mut games = Vec::new();
    let mut start = None;

    for (idx, _) in text.match_indices("[Event ") {
        if let Some(prev) = start {
            let game = text[prev..idx].trim();
            if !game.is_empty() {
                games.push(game);
            }
        }
        start = Some(idx);
    }

    if let Some(prev) = start {
        let game = text[prev..].trim();
        if !game.is_empty() {
            games.push(game);
        }
    }

    games
}

fn parse_headers_and_moves(game: &str) -> io::Result<(GameResult, &str)> {
    let mut result = None;
    let mut offset = 0usize;
    let in_headers = true;

    for line in game.lines() {
        let trimmed = line.trim();

        if in_headers && trimmed.starts_with("[") {
            if let Some(tag) = trimmed.strip_prefix("[Result \"") {
                if let Some(tag) = tag.strip_suffix("\"]") {
                    result = GameResult::from_pgn_tag(tag);
                }
            }
        } else if trimmed.is_empty() && in_headers {
            // Blank line
        } else {
            let moves = game[offset..].trim();
            let result = result.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Missing or invalid Result tag")
            })?;
            return Ok((result, moves));
        }

        offset += line.len() + 1;
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Game has headers but no movetext",
    ))
}

fn is_move_number(token: &str) -> bool {
    token.ends_with('.')
        || token.ends_with("...")
        || token.chars().all(|c| c.is_ascii_digit() || c == '.')
}

#[derive(Clone, Debug)]
struct MoveToken {
    san: String,
    eval: Option<CommentEval>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum CommentEval {
    Cp(i32),
    Mate,
}

fn parse_comment_eval(comment: &str) -> Option<CommentEval> {
    let body = comment.trim();
    let head = body.split_ascii_whitespace().next()?; // "+0.17/7" or "+320.00/19"
    let score_text = head.split_once('/')?.0;
    let pawns: f64 = score_text.parse().ok()?;
    let cp = (pawns * 100.0).round() as i32;

    if body.contains(" mates") || cp.abs() >= 31_001 {
        Some(CommentEval::Mate)
    } else {
        Some(CommentEval::Cp(cp))
    }
}

fn is_result_token(token: &str) -> bool {
    matches!(token, "1-0" | "0-1" | "1/2-1/2" | "*")
}

fn push_token(buf: &mut String, out: &mut Vec<MoveToken>) {
    let token = buf.trim_end_matches(['!', '?']).trim();
    if !token.is_empty()
        && !is_move_number(token)
        && !token.starts_with('$')
        && !is_result_token(token)
    {
        out.push(MoveToken {
            san: token.to_string(),
            eval: None,
        });
    }
    buf.clear();
}

fn tokenize_moves(movetext: &str) -> io::Result<Vec<MoveToken>> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut chars = movetext.chars().peekable();
    let mut variation_depth = 0usize;
    let mut semicolon_comment = false;

    while let Some(ch) = chars.next() {
        if semicolon_comment {
            if ch == '\n' {
                semicolon_comment = false;
            }
            continue;
        }

        if variation_depth > 0 {
            match ch {
                '(' => variation_depth += 1,
                ')' => variation_depth -= 1,
                _ => {}
            }
            continue;
        }

        match ch {
            '{' => {
                push_token(&mut buf, &mut out);

                let mut comment = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '}' {
                        closed = true;
                        break;
                    }
                    comment.push(c);
                }

                if !closed {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unterminated PGN comment",
                    ));
                }

                if let Some(last) = out.last_mut() {
                    last.eval = parse_comment_eval(comment.trim());
                }
            }
            '(' => {
                push_token(&mut buf, &mut out);
                variation_depth = 1;
            }
            ';' => {
                push_token(&mut buf, &mut out);
                semicolon_comment = true;
            }
            c if c.is_whitespace() => {
                push_token(&mut buf, &mut out);
            }
            _ => buf.push(ch),
        }
    }

    push_token(&mut buf, &mut out);
    Ok(out)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct SanSpec {
    piece: Piece,
    to: Square,
    capture: bool,
    promotion: Option<Piece>,
    from_file: Option<u8>,
    from_rank: Option<u8>,
}

fn san_to_move(pos: &Position, san: &str) -> io::Result<Move> {
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

fn keep_position(ply: usize) -> bool {
    ply > 12
}

pub fn parse_games(path: &Path, out: &Path) -> io::Result<()> {
    let text = fs::read_to_string(path)?;
    let mut writer = BufWriter::new(File::create(out)?);

    for game in split_games(&text) {
        let (result, move_text) = parse_headers_and_moves(game)?;
        let mut pos = Position::default();

        for (ply, token) in tokenize_moves(move_text)?.into_iter().enumerate() {
            // Skip mating scores
            if let Some(CommentEval::Mate) = token.eval {
                break;
            }

            let mv = san_to_move(&pos, &token.san)?;
            let mut state = StateInfo::new();

            if keep_position(ply + 1) {
                writeln!(writer, "{} \"{}\";", pos.to_fen(), result)?;
            }

            pos.make_move(mv, &mut state);
        }
    }

    Ok(())
}

#[derive(Copy, Clone)]
struct Sample {
    pos: Position,
    result: GameResult,
}

use crate::evaluate::Score;

fn fmt_score(s: Score) -> String {
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

fn _load_samples(path: &Path) -> io::Result<Vec<Sample>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut samples = Vec::new();

    for line in reader.lines() {
        let line: String = line?;
        if line.trim().is_empty() {
            continue;
        }

        let (fen, result_part) = line.rsplit_once(" \"").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, format!("bad line: {line}"))
        })?;

        let result_str = result_part.strip_suffix("\";").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("bad result suffix: {line}"),
            )
        })?;

        let pos = Position::from_fen(fen);
        let result = GameResult::from_pgn_tag(result_str).unwrap();

        samples.push(Sample { pos, result })
    }

    Ok(samples)
}

fn load_epd_samples(path: &Path) -> io::Result<Vec<Sample>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut samples = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.splitn(5, ' ');
        let board = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, line))?;
        let stm = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, line))?;
        let castling = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, line))?;
        let ep = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, line))?;
        let ops = parts
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, line))?;

        let result_start = ops
            .find('"')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, line))?;
        let result_end = ops[result_start + 1..]
            .find('"')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, line))?
            + result_start
            + 1;

        let result_str = &ops[result_start + 1..result_end];
        let result = GameResult::from_pgn_tag(result_str)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, line))?;

        let fen = format!("{board} {stm} {castling} {ep} 0 1");
        let pos = Position::from_fen(&fen);

        samples.push(Sample { pos, result });
    }

    Ok(samples)
}

fn texel_qsearch(pos: &mut Position, params: &Params, mut alpha: Eval, beta: Eval) -> Eval {
    if pos.halfmove_clock >= 100 || pos.insufficient_material() {
        return 0;
    }

    let in_check = !pos.checkers.is_empty();

    let mut best_score = if in_check {
        -INFINITY
    } else {
        let stand_pat = evaluate_with(pos, params);

        if stand_pat >= beta {
            return stand_pat;
        }

        alpha = alpha.max(stand_pat);
        stand_pat
    };

    let mut moves = if in_check {
        generate_legal(pos, &mut MoveList::new())
    } else {
        generate_legal_noisy(pos, &mut MoveList::new())
    };

    let ordering = OrderingTables::new();
    moves
        .as_mut_slice()
        .sort_unstable_by_key(|&mv| Reverse(ordering.score_noisy(pos, mv)));

    for &mv in moves.as_slice() {
        let mut state = StateInfo::new();
        pos.make_move(mv, &mut state);

        let score = -texel_qsearch(pos, params, -beta, -alpha);

        pos.undo_move(mv, &state);

        if score >= beta {
            return score;
        }

        best_score = best_score.max(score);
        alpha = alpha.max(score);
    }

    best_score
}

fn texel_root_qsearch(pos: &mut Position, params: &Params) -> Eval {
    let score = texel_qsearch(pos, params, -INFINITY, INFINITY);

    return if pos.side_to_move == Colour::White {
        score
    } else {
        -score
    };
}

fn sigmoid(k: f64, qscore: i32) -> f64 {
    1.0 / (1.0 + 10.0_f64.powf(-k * qscore as f64 / 400.0)) as f64
}

fn sample_error(sample: &Sample, params: &Params, k: f64) -> f64 {
    let mut pos = sample.pos;
    let result = sample.result.to_value();

    let qscore = texel_root_qsearch(&mut pos, &params);

    // Calculate evaluation error
    let diff = result - sigmoid(k, qscore);
    diff * diff
}

fn loss(samples: &[Sample], params: &Params, k: f64) -> f64 {
    let total: f64 = samples.par_iter().map(|s| sample_error(s, params, k)).sum();

    total / samples.len() as f64
}

fn _fit_k(samples: &[Sample], params: &Params) -> f64 {
    let mut best_k = 0.1;
    let mut best_loss = f64::INFINITY;

    let mut k = 0.0;
    while k <= 2.0 {
        let err = loss(samples, params, k);
        if err < best_loss {
            best_loss = err;
            best_k = k;
        }
        k += 0.001;
    }

    best_k
}

fn calibrate_a(
    samples: &[Sample],
    theta: &[i32],
    k: f64,
    c: f64,
    a_cap: f64,
    alpha: f64,
    desired_first_step: f64,
    bounds: &[ParamBounds],
) -> f64 {
    let trials = 8;
    let mut grad_sum = 0.0;
    let mut grad_n = 0usize;

    for _ in 0..trials {
        let delta: Vec<i32> = (0..theta.len())
            .map(|_| if rand::random::<bool>() { 1 } else { -1 })
            .collect();

        let mut plus = theta.to_vec();
        let mut minus = theta.to_vec();

        let c_i = c.round() as i32;

        for i in 0..theta.len() {
            plus[i] = (plus[i] + c_i * delta[i]).clamp(bounds[i].min, bounds[i].max);
            minus[i] = (minus[i] - c_i * delta[i]).clamp(bounds[i].min, bounds[i].max);
        }

        let loss_plus = loss(samples, &Params::unpack(&plus), k);
        let loss_minus = loss(samples, &Params::unpack(&minus), k);

        for i in 0..theta.len() {
            let g_i = (loss_plus - loss_minus) / (2.0 * c * delta[i] as f64);
            grad_sum += g_i.abs();
            grad_n += 1;
        }
    }

    let mean_abs_grad = grad_sum / grad_n as f64;

    if mean_abs_grad == 0.0 {
        return 0.1;
    }

    desired_first_step * (a_cap + 1.0).powf(alpha) / mean_abs_grad
}

fn build_thread_pool() {
    let threads = std::env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(16);

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .unwrap();
}

fn make_nondecreasing<const N: usize>(arr: &mut [Score; N]) {
    for i in 1..N {
        arr[i].mg = arr[i].mg.max(arr[i - 1].mg);
        arr[i].eg = arr[i].eg.max(arr[i - 1].eg);
    }
}

fn make_nonincreasing<const N: usize>(arr: &mut [Score; N]) {
    for i in 1..N {
        arr[i].mg = arr[i].mg.min(arr[i - 1].mg);
        arr[i].eg = arr[i].eg.min(arr[i - 1].eg);
    }
}

fn normalise_mean_zero<const N: usize>(base: &mut Score, arr: &mut [Score; N]) {
    let mean_mg = arr.iter().map(|s| s.mg).sum::<i32>() / N as i32;
    let mean_eg = arr.iter().map(|s| s.eg).sum::<i32>() / N as i32;

    for s in arr {
        s.mg -= mean_mg;
        s.eg -= mean_eg;
    }

    base.mg += mean_mg;
    base.eg += mean_eg;
}

fn project_params(params: &mut Params) {
    make_nondecreasing(&mut params.passed_pawn);

    make_nondecreasing(&mut params.knight_adj);
    make_nonincreasing(&mut params.rook_adj);

    params.tripled_pawns.mg = params.tripled_pawns.mg.min(params.doubled_pawns.mg);
    params.tripled_pawns.eg = params.tripled_pawns.eg.min(params.doubled_pawns.eg);

    params.quadrupled_pawns.mg = params.quadrupled_pawns.mg.min(params.tripled_pawns.mg);
    params.quadrupled_pawns.eg = params.quadrupled_pawns.eg.min(params.tripled_pawns.eg);

    // Material value
    params.bishop_value.mg = params.bishop_value.mg.max(params.knight_value.mg - 20);
    params.bishop_value.eg = params.bishop_value.eg.max(params.knight_value.eg - 20);

    params.rook_value.mg = params.rook_value.mg.max(params.bishop_value.mg + 100);
    params.rook_value.eg = params.rook_value.eg.max(params.bishop_value.eg + 100);

    // Normalise knight/rook values from adjustment tables
    // Results in knight/rook piece values more accurately representing their true value
    normalise_mean_zero(&mut params.knight_value, &mut params.knight_adj);
    normalise_mean_zero(&mut params.rook_value, &mut params.rook_adj);
}

fn dump_params(label: &str, theta: &[i32], loss: f64) {
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
        "pub const KING_RING_ATTACKS: [Score; 5] = {};",
        fmt_score_array(&params.king_ring_attacks)
    );
    println!(
        "pub const ENEMY_PAWN_DISTANCE_FROM_BACKRANK: [Score; 4] = {};",
        fmt_score_array(&params.enemy_pawn_distance_from_backrank)
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

pub fn tune_params(path: &Path) {
    build_thread_pool();

    // let samples = load_samples(path).unwrap();
    let samples = load_epd_samples(path).unwrap();

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = Arc::clone(&stop);
        ctrlc::set_handler(move || {
            stop.store(true, Ordering::Relaxed);
        })
        .expect("failed to install Ctrl+C handler");
    }

    let iterations = 50_000;
    let alpha = 0.602;
    let gamma = 0.101;
    let c = 2.0;
    let A = 0.1 * iterations as f64;

    // Fitted k value
    // let k = _fit_k(&samples, &DEFAULT_PARAMS);
    let k = 1.377;
    println!("fitted k value: {k}");

    let theta = DEFAULT_PARAMS.pack();
    let bounds = Params::flat_bounds();
    debug_assert_eq!(theta.len(), bounds.len());

    let mut params = DEFAULT_PARAMS;
    project_params(&mut params);
    let mut theta = params.pack();

    let a = calibrate_a(&samples, &theta, k, c, A, alpha, 1.0, &bounds);
    println!("a value: {a}");

    let baseline_loss = loss(&samples, &params, k);

    let mut best_theta = theta.clone();
    let mut current_loss = baseline_loss;
    let mut best_loss = baseline_loss;

    for t in 0..iterations {
        if stop.load(Ordering::Relaxed) {
            println!("Interrupted at iter={t}");
            break;
        }

        let a_t = a / (t as f64 + 1.0 + A).powf(alpha);
        let c_t = c / (t as f64 + 1.0).powf(gamma);

        let delta: Vec<i32> = (0..theta.len())
            .map(|_| if rand::random::<bool>() { 1 } else { -1 })
            .collect();

        let mut plus = theta.clone();
        let mut minus = theta.clone();

        for i in 0..theta.len() {
            plus[i] += (c_t.round() as i32) * delta[i];
            minus[i] -= (c_t.round() as i32) * delta[i];
            plus[i] = plus[i].clamp(bounds[i].min, bounds[i].max);
            minus[i] = minus[i].clamp(bounds[i].min, bounds[i].max);
        }

        // Apply project to theta to ensure values are logical
        let mut params = Params::unpack(&theta);
        let mut params_plus = Params::unpack(&plus);
        let mut params_minus = Params::unpack(&minus);
        project_params(&mut params);
        project_params(&mut params_plus);
        project_params(&mut params_minus);
        plus = params_plus.pack();
        minus = params_minus.pack();
        theta = params.pack();

        let loss_plus = loss(&samples, &Params::unpack(&plus), k);
        if stop.load(Ordering::Relaxed) {
            println!("Interrupted at iter={t} after loss_plus");
            break;
        }

        let loss_minus = loss(&samples, &Params::unpack(&minus), k);
        if stop.load(Ordering::Relaxed) {
            println!("Interrupted at iter={t} after loss_minus");
            break;
        }

        for i in 0..theta.len() {
            let g_i = (loss_plus - loss_minus) / (2.0 * c_t * delta[i] as f64);
            theta[i] = (theta[i] as f64 - a_t * g_i).round() as i32;
            theta[i] = theta[i].clamp(bounds[i].min, bounds[i].max);
        }

        // Apply project to theta to ensure values are logical
        let mut params = Params::unpack(&theta);
        project_params(&mut params);
        theta = params.pack();

        current_loss = loss(&samples, &params, k);

        if current_loss < best_loss {
            best_loss = current_loss;
            best_theta = theta.clone();
        }

        if t % 100 == 0 {
            println!(
                "pawn: {}, knight: {}, bishop: {}, rook: {}, queen: {}",
                fmt_score(params.pawn_value),
                fmt_score(params.knight_value),
                fmt_score(params.bishop_value),
                fmt_score(params.rook_value),
                fmt_score(params.queen_value),
            );
            println!("iter={t}, current_loss={current_loss}, best_loss={best_loss}");
        }
    }

    dump_params("current", &theta, current_loss);
    dump_params("best", &best_theta, best_loss);
}
