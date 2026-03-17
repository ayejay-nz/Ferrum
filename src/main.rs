use rust_engine::{position, search};

fn main() {
    let mut pos = position::Position::default();

    search::search(&mut pos, 6);
}
