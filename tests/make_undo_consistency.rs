use rust_engine::bitboard::Bitboards;
use rust_engine::position::{DEFAULT_FEN, Position, StateInfo};
use rust_engine::types::{Move, MoveFlag, Square};

#[test]
fn make_undo_move_loop() {
    let bbs = Bitboards::init();
    let mut pos = Position::from_fen(DEFAULT_FEN);
    let mut state = StateInfo::new();
    state.set_from_position(&pos);

    const HALFMOVE_COUNT: usize = 24;
    let move_sequence: [Move; HALFMOVE_COUNT] = [
        Move::new(Square::D2, Square::D4, MoveFlag::DoublePush),
        Move::new(Square::D7, Square::D5, MoveFlag::DoublePush),
        Move::new(Square::C2, Square::C4, MoveFlag::DoublePush),
        Move::new(Square::E7, Square::E6, MoveFlag::Quiet),
        Move::new(Square::B1, Square::C3, MoveFlag::Quiet),
        Move::new(Square::G8, Square::F6, MoveFlag::Quiet),
        Move::new(Square::C1, Square::G5, MoveFlag::Quiet),
        Move::new(Square::F8, Square::E7, MoveFlag::Quiet),
        Move::new(Square::E2, Square::E3, MoveFlag::Quiet),
        Move::new(Square::H7, Square::H6, MoveFlag::Quiet),
        Move::new(Square::G5, Square::F6, MoveFlag::Capture),
        Move::new(Square::E7, Square::F6, MoveFlag::Capture),
        Move::new(Square::G1, Square::F3, MoveFlag::Quiet),
        Move::new(Square::E8, Square::G8, MoveFlag::KingCastle),
        Move::new(Square::C4, Square::D5, MoveFlag::Capture),
        Move::new(Square::C7, Square::C5, MoveFlag::DoublePush),
        Move::new(Square::D5, Square::C6, MoveFlag::EpCapture),
        Move::new(Square::B7, Square::B5, MoveFlag::DoublePush),
        Move::new(Square::C6, Square::C7, MoveFlag::Quiet),
        Move::new(Square::B8, Square::C6, MoveFlag::Quiet),
        Move::new(Square::C7, Square::D8, MoveFlag::PromoCaptureQ),
        Move::new(Square::F8, Square::D8, MoveFlag::Capture),
        Move::new(Square::F1, Square::B5, MoveFlag::Capture),
        Move::new(Square::C6, Square::E7, MoveFlag::Quiet),
    ];

    let mut positions = [Position::new(); HALFMOVE_COUNT];
    let mut states = [StateInfo::new(); HALFMOVE_COUNT];

    // Find every position and state for the entire move sequence
    for (i, mv) in move_sequence.iter().enumerate() {
        positions[i] = pos;
        pos.make_move(*mv, &mut state, &bbs);
        states[i] = state;
    }

    // Compare undone positions to real versions
    for (i, mv) in move_sequence.iter().enumerate().rev() {
        pos.undo_move(*mv, &states[i], &bbs);
        assert_eq!(pos, positions[i]);
    }
}
