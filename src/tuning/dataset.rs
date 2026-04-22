use std::{
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind, Result},
    path::Path,
};

use crate::{
    position::Position,
    tuning::types::{GameResult, Sample},
};

pub fn load_samples(path: &Path) -> Result<Vec<Sample>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut samples = Vec::new();

    for line in reader.lines() {
        let line: String = line?;
        if line.trim().is_empty() {
            continue;
        }

        let (fen, result_part) = line
            .rsplit_once(" \"")
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("bad line: {line}")))?;

        let result_str = result_part.strip_suffix("\";").ok_or_else(|| {
            Error::new(ErrorKind::InvalidData, format!("bad result suffix: {line}"))
        })?;

        let pos = Position::from_fen(fen);
        let result = GameResult::from_pgn_tag(result_str)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, line))?;

        samples.push(Sample { pos, result })
    }

    Ok(samples)
}

pub fn load_epd_samples(path: &Path) -> Result<Vec<Sample>> {
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
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, line))?;
        let stm = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, line))?;
        let castling = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, line))?;
        let ep = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, line))?;
        let ops = parts
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, line))?;

        let result_start = ops
            .find('"')
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, line))?;
        let result_end = ops[result_start + 1..]
            .find('"')
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, line))?
            + result_start
            + 1;

        let result_str = &ops[result_start + 1..result_end];
        let result = GameResult::from_pgn_tag(result_str)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, line))?;

        let fen = format!("{board} {stm} {castling} {ep} 0 1");
        let pos = Position::from_fen(&fen);

        samples.push(Sample { pos, result });
    }

    Ok(samples)
}
