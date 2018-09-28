extern crate pgn_reader;
extern crate shakmaty;

use pgn_reader::{Visitor, Skip, Reader, San};

struct MoveCounter {
    moves: usize,
}

impl MoveCounter {
    fn new() -> MoveCounter {
        MoveCounter { moves: 0 }
    }
}

impl<'pgn> Visitor<'pgn> for MoveCounter {
    type Result = usize;

    fn begin_game(&mut self) {
        self.moves = 0;
    }

    fn san(&mut self, san: San) {
        println!("{:?}", san);
        self.moves += 1;
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn end_game(&mut self, _game: &'pgn [u8]) -> Self::Result {
        self.moves
    }
}

fn main() {
    let pgn = b"e4 e5 Nf3 Nf6 Nf4=Q Re4xd5+";
    println!("{:?}", std::str::from_utf8(pgn));

    let mut counter = MoveCounter::new();
    let reader = Reader::new(&mut counter, pgn);

    let moves: usize = reader.into_iter().sum();
    assert_eq!(moves, 6);
}

