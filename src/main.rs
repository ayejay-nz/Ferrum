mod position;
mod types;
mod zobrist;

fn main() {
    let pos = position::Position::from_fen(position::DEFAULT_FEN);
    pos.display();
}
