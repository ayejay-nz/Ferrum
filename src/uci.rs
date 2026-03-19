use std::{
    io::{self, BufRead, Write},
    time::Duration,
};

use crate::{
    movegen::{MoveList, generate_legal},
    position::{Position, StateInfo},
    search::{self, SearchLimits},
    types::{Colour, Move},
};

fn parse_uci_move(pos: &Position, text: &str) -> Option<Move> {
    let moves = generate_legal(pos, &mut MoveList::new());

    moves
        .as_slice()
        .iter()
        .copied()
        .find(|&mv| mv.to_string() == text)
}

fn set_position(pos: &mut Position, command: &str) {
    // UCI format:
    // position startpos [moves ...]
    // position fen <fen-string> [moves ...]

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

    if parts.next() == Some("moves") {
        for mv_text in parts {
            let Some(mv) = parse_uci_move(pos, mv_text) else {
                return;
            };

            let mut state = StateInfo::new();
            pos.make_move(mv, &mut state);
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

pub fn run() {
    let stdin = io::stdin();
    let mut pos = Position::default();

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        if line == "uci" {
            println!("id name rust_engine");
            println!("id author ayejay");
            println!("uciok");
        } else if line == "isready" {
            println!("readyok");
        } else if line == "ucinewgame" {
            pos = Position::default();
        } else if line.starts_with("position ") {
            set_position(&mut pos, &line);
        } else if line.starts_with("go") {
            let limits = SearchLimits {
                max_depth: parse_go_depth(&line).unwrap_or(64),
                move_time: parse_go_movetime(&line).or_else(|| parse_go_clock_time(&line, &pos)),
            };
            let result = search::search(&mut pos, limits);
            println!("bestmove {}", result.best_move);
        } else if line == "quit" {
            break;
        }

        io::stdout().flush().unwrap();
    }
}
