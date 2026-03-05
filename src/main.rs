mod position;
mod types;

fn main() {
    let pos = position::Position::load_fen(position::DEFAULT_FEN);
    pos.display();
}
