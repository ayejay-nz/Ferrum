use rust_engine::position;

fn main() {
    let pos = position::Position::from_fen(position::DEFAULT_FEN);
    pos.display();
}
