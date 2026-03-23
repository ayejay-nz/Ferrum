use std::{
    io::{self, BufRead, Write},
    time::{Duration, Instant},
};

use crate::{
    movegen::{MoveList, generate_legal},
    position::{Position, StateInfo},
    search::{self, SearchContext, SearchLimits, SearchResult},
    tt::TranspositionTable,
    types::{Colour, Move, Piece},
    zobrist::ZKey,
};

#[derive(Copy, Clone)]
struct EngineOptions {
    use_book: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self { use_book: true }
    }
}

fn parse_setoption_use_book(command: &str) -> Option<bool> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.first().copied() != Some("setoption") {
        return None;
    }

    let name_idx = parts.iter().position(|&p| p == "name")?;
    let value_idx = parts.iter().position(|&p| p == "value")?;

    let name = parts[name_idx + 1..value_idx].join(" ");
    if name != "UseBook" {
        return None;
    }

    match parts.get(value_idx + 1)?.to_ascii_lowercase().as_str() {
        "true" | "1" | "on" => Some(true),
        "false" | "0" | "off" => Some(false),
        _ => None,
    }
}

fn parse_uci_move(pos: &Position, text: &str) -> Option<Move> {
    let moves = generate_legal(pos, &mut MoveList::new());

    moves
        .as_slice()
        .iter()
        .copied()
        .find(|&mv| mv.to_string() == text)
}

fn set_position(pos: &mut Position, history: &mut Vec<ZKey>, command: &str) {
    // UCI format:
    // position startpos [moves ...]
    // position fen <fen-string> [moves ...]

    history.clear();
    let mut parts = command.split_whitespace();

    if parts.next() != Some("position") {
        return;
    }

    match parts.next() {
        Some("startpos") => {
            *pos = Position::default();
        }
        Some("fen") => {
            let fen_parts: Vec<&str> = parts.by_ref().take(6).collect();
            if fen_parts.len() != 6 {
                return;
            }
            *pos = Position::from_fen(&fen_parts.join(" "));
        }
        _ => return,
    }

    history.push(pos.zkey);

    if parts.next() == Some("moves") {
        for mv_text in parts {
            let Some(mv) = parse_uci_move(pos, mv_text) else {
                return;
            };

            let irreversible =
                mv.is_capture() || pos.mailbox.piece_at(mv.from()) == Some(Piece::Pawn);

            let mut state = StateInfo::new();
            pos.make_move(mv, &mut state);

            if irreversible {
                history.clear();
            }
            history.push(pos.zkey);
        }
    }
}

fn parse_go_depth(command: &str) -> Option<i32> {
    let mut parts = command.split_whitespace();

    if parts.next() != Some("go") {
        return None;
    }

    while let Some(part) = parts.next() {
        if part == "depth" {
            let depth = parts.next()?.parse::<i32>().ok()?;
            return (depth > 0).then_some(depth);
        }
    }

    None
}

fn parse_go_movetime(command: &str) -> Option<Duration> {
    let mut parts = command.split_whitespace();

    if parts.next() != Some("go") {
        return None;
    }

    while let Some(part) = parts.next() {
        if part == "movetime" {
            let ms = parts.next()?.parse::<u64>().ok()?;
            return Some(Duration::from_millis(ms));
        }
    }

    None
}

fn parse_go_u64(command: &str, key: &str) -> Option<u64> {
    let mut parts = command.split_whitespace();

    if parts.next() != Some("go") {
        return None;
    }

    while let Some(part) = parts.next() {
        if part == key {
            return parts.next()?.parse::<u64>().ok();
        }
    }

    None
}

fn parse_go_clock_time(command: &str, pos: &Position) -> Option<Duration> {
    let (time_key, inc_key) = match pos.side_to_move {
        Colour::White => ("wtime", "winc"),
        Colour::Black => ("btime", "binc"),
    };

    let time_ms = parse_go_u64(command, time_key)?;
    let inc_ms = parse_go_u64(command, inc_key).unwrap_or(0);

    // Spend 1/20 of remaining time plus half the increment
    let think_ms = (time_ms / 20).saturating_add(inc_ms / 2);

    // Small safety buffer
    Some(Duration::from_millis(think_ms.saturating_sub(10).max(5)))
}

pub fn emit_uci_info(result: &SearchResult, ctx: &SearchContext, start: Instant) {
    let time_ms = start.elapsed().as_millis().max(1) as u64;
    let nps = ctx.stats.nodes.saturating_mul(1000) / time_ms;

    println!(
        "info depth {} score cp {} time {} nodes {} nps {} pv {}",
        result.depth, result.score, time_ms, ctx.stats.nodes, nps, result.best_move,
    );

    io::stdout().flush().unwrap();
}

pub fn run() {
    let stdin = io::stdin();
    let mut pos = Position::default();
    let mut tt = TranspositionTable::new(32);
    let mut history: Vec<ZKey> = Vec::with_capacity(128);
    history.push(pos.zkey);

    let mut options = EngineOptions::default();

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if line == "uci" {
            println!("id name rust_engine");
            println!("id author ayejay");
            println!("option name UseBook type check default true");
            println!("uciok");
        } else if line == "isready" {
            println!("readyok");
        } else if line == "ucinewgame" {
            pos = Position::default();
            tt.clear();
            history.clear();
            history.push(pos.zkey);
        } else if line.starts_with("position ") {
            set_position(&mut pos, &mut history, &line);
        } else if line.starts_with("go") {
            let limits = SearchLimits {
                max_depth: parse_go_depth(&line).unwrap_or(64),
                move_time: parse_go_movetime(&line).or_else(|| parse_go_clock_time(&line, &pos)),
            };
            let result = search::search(&mut pos, &mut tt, &history, limits, options.use_book);
            println!("bestmove {}", result.best_move);
        } else if line.starts_with("setoption ") {
            if let Some(use_book) = parse_setoption_use_book(&line) {
                options.use_book = use_book;
            }
        } else if line == "quit" {
            break;
        }

        io::stdout().flush().unwrap();
    }
}
