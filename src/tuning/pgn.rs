use std::{
    fs::{File, read_to_string},
    io::{BufWriter, Error, ErrorKind, Result, Write},
    path::Path,
};

use crate::{
    position::{Position, StateInfo},
    tuning::{san::san_to_move, types::GameResult},
};

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

fn keep_position(ply: usize) -> bool {
    ply > 12
}

pub fn parse_games(path: &Path, out: &Path) -> Result<()> {
    let text = read_to_string(path)?;
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

fn parse_headers_and_moves(game: &str) -> Result<(GameResult, &str)> {
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
                Error::new(ErrorKind::InvalidData, "Missing or invalid Result tag")
            })?;
            return Ok((result, moves));
        }

        offset += line.len() + 1;
    }

    Err(Error::new(
        ErrorKind::InvalidData,
        "Game has headers but no movetext",
    ))
}

fn is_move_number(token: &str) -> bool {
    token.ends_with('.')
        || token.ends_with("...")
        || token.chars().all(|c| c.is_ascii_digit() || c == '.')
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

fn tokenize_moves(movetext: &str) -> Result<Vec<MoveToken>> {
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
                    return Err(Error::new(
                        ErrorKind::InvalidData,
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
