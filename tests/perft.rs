use ferrum::{
    movegen::{MoveList, generate_legal},
    position::{Position, StateInfo},
};

fn perft(pos: &mut Position, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal(pos, &mut MoveList::new());

    let mut nodes = 0;
    let mut state = StateInfo::new();

    for &mv in moves.as_slice() {
        pos.make_move(mv, &mut state);
        nodes += perft(pos, depth - 1);
        pos.undo_move(mv, &state);
    }

    nodes
}

#[test]
fn perft_is_correct() {
    // Test move gen against known perft values
    const POSITIONS: usize = 6;
    const PERFT_DEPTHS: [usize; POSITIONS] = [5, 5, 6, 5, 5, 5];
    const PERFT_VALUES: [[u64; 6]; POSITIONS] = [
        [1, 20, 400, 8_902, 197_281, 4_865_609],
        [1, 48, 2_039, 97_862, 4_085_603, 193_690_690],
        [1, 14, 191, 2_812, 43_238, 674_624],
        [1, 6, 264, 9_467, 422_333, 15_833_292],
        [1, 44, 1_486, 62_379, 2_103_487, 89_941_194],
        [1, 46, 2_079, 89_890, 3_894_594, 164_075_551],
    ];
    const PERFT_FENS: [&str; POSITIONS] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    ];

    for p in 0..POSITIONS {
        let fen = PERFT_FENS[p];
        let depth = PERFT_DEPTHS[p];
        let values = PERFT_VALUES[p];

        let mut pos = Position::from_fen(fen);
        for d in 0..depth {
            let value = values[d];
            let calculated = perft(&mut pos, d);

            assert_eq!(value, calculated);
        }

        println!("Perft position {} passed!", p + 1);
    }
}
