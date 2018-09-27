#[macro_use]
pub extern crate enum_primitive;
#[macro_use]
extern crate failure;

pub type Result<T> = std::result::Result<T, failure::Error>;

use enum_primitive::FromPrimitive;

#[derive(Debug, Clone)]
pub struct BoardState {
    pieces: [[Piece; 8]; 8],
    pub current_player: CurrentPlayer,
}

#[derive(Debug, Copy, PartialEq, Eq, Clone)]
pub enum CurrentPlayer {
    White,
    Black,
}

impl BoardState {
    pub fn init() -> BoardState {
        BoardState {
            pieces: [
                [
                    Piece::WhiteRook,
                    Piece::WhiteKnight,
                    Piece::WhiteBishop,
                    Piece::WhiteKing,
                    Piece::WhiteQueen,
                    Piece::WhiteBishop,
                    Piece::WhiteKnight,
                    Piece::WhiteRook,
                ],
                [
                    Piece::WhitePawn,
                    Piece::WhitePawn,
                    Piece::WhitePawn,
                    Piece::WhitePawn,
                    Piece::WhitePawn,
                    Piece::WhitePawn,
                    Piece::WhitePawn,
                    Piece::WhitePawn,
                ],
                [
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                ],
                [
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                ],
                [
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                ],
                [
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                    Piece::None,
                ],
                [
                    Piece::BlackPawn,
                    Piece::BlackPawn,
                    Piece::BlackPawn,
                    Piece::BlackPawn,
                    Piece::BlackPawn,
                    Piece::BlackPawn,
                    Piece::BlackPawn,
                    Piece::BlackPawn,
                ],
                [
                    Piece::BlackRook,
                    Piece::BlackKnight,
                    Piece::BlackBishop,
                    Piece::BlackKing,
                    Piece::BlackQueen,
                    Piece::BlackBishop,
                    Piece::BlackKnight,
                    Piece::BlackRook,
                ],
            ],
            current_player: CurrentPlayer::White,
        }
    }

    fn get_position(m: &str) -> (u8, u8) {
        let mut offset = 0;
        let bytes = m.as_bytes();
        while bytes[offset] == b'x' {
            offset += 1;
        }
        let x = bytes[offset];
        let y = bytes[offset + 1];
        if x > b'h' || x < b'a' {
            panic!("X out of bounds: {:?}", x as char);
        }
        if y > b'8' || y < b'1' {
            panic!("Y out of bounds: {:?}", y as char);
        }
        (7 - (x - b'a'), y - b'1')
    }

    #[inline]
    pub fn get_piece(&self, x: u8, y: u8) -> Piece {
        self.pieces[y as usize][x as usize]
    }

    fn move_piece(&mut self, from: (u8, u8), to: (u8, u8)) {
        let mut piece = self.pieces[from.1 as usize][from.0 as usize];
        assert!(piece != Piece::None);
        piece.has_moved();
        self.pieces[from.1 as usize][from.0 as usize] = Piece::None;
        self.pieces[to.1 as usize][to.0 as usize] = piece;

        self.current_player = match self.current_player {
            CurrentPlayer::White => CurrentPlayer::Black,
            CurrentPlayer::Black => CurrentPlayer::White,
        };
    }

    fn set_piece(&mut self, position: (u8, u8), piece: Piece) {
        self.pieces[position.1 as usize][position.0 as usize] = piece;
    }

    fn find_piece(
        &mut self,
        tile: (u8, u8),
        movements: &[(i8, i8)],
        expected: &[Piece],
    ) -> (Piece, u8, u8) {
        for movement in movements {
            let mut position = tile;
            loop {
                let next_position = (position.0 as i8 + movement.0, position.1 as i8 + movement.1);
                if next_position.0 < 0 || next_position.0 > 7 {
                    break;
                }
                if next_position.1 < 0 || next_position.1 > 7 {
                    break;
                }
                position = (next_position.0 as u8, next_position.1 as u8);

                let piece = self.get_piece(position.0, position.1);
                for e in expected {
                    if piece == *e {
                        return (*e, position.0, position.1);
                    }
                }
                if piece != Piece::None {
                    break;
                }
            }
        }
        panic!("Could not find requested piece");
    }

    fn move_pawn(&mut self, to: &str) -> Result<()> {
        let (x, y) = BoardState::get_position(to);
        match self.current_player {
            CurrentPlayer::White => {
                // find pawns below this
                for check_y in (y - 2..y).rev() {
                    if self.get_piece(x, check_y) == Piece::WhitePawn
                        || self.get_piece(x, check_y) == Piece::WhitePawnMoved
                    {
                        self.move_piece((x, check_y), (x, y));
                        self.try_promote_pawn((x, y), to);
                        return Ok(());
                    }
                }
                bail!("Could not find pawn to move to {:?}", to);
            }
            CurrentPlayer::Black => {
                // find pawns above this
                for check_y in y + 1..y + 3 {
                    if self.get_piece(x, check_y) == Piece::BlackPawn
                        || self.get_piece(x, check_y) == Piece::BlackPawnMoved
                    {
                        self.move_piece((x, check_y), (x, y));
                        self.try_promote_pawn((x, y), to);
                        return Ok(());
                    }
                }
                bail!("Could not find pawn to move to {:?}", to);
            }
        }
    }

    fn try_promote_pawn(&mut self, target: (u8, u8), command: &str) {
        if target.1 == 7 || target.1 == 0 {
            let bytes = command.as_bytes();
            let mut space_position = bytes.len() - 2;
            while space_position != 0 {
                if bytes[space_position] == b'=' {
                    break;
                }
                space_position -= 1;
            }
            assert_eq!(b'=', bytes[space_position]);
            match bytes[space_position + 1] {
                b'Q' => self.set_piece(
                    target,
                    match target.1 {
                        7 => Piece::WhiteQueen,
                        0 => Piece::BlackQueen,
                        _ => unreachable!(),
                    },
                ),
                _ => unimplemented!(),
            }
        }
    }

    fn capture_with_pawn(&mut self, target: &str, column: &str) -> Result<()> {
        let (x, y) = BoardState::get_position(target);
        let (source_x, _) = BoardState::get_position(&format!("{}1", column));
        let source_y = match self.current_player {
            CurrentPlayer::White => y - 1,
            CurrentPlayer::Black => y + 1,
        };
        if Piece::None == self.get_piece(x, y) {
            let target_y = match self.current_player {
                CurrentPlayer::White => y - 1,
                CurrentPlayer::Black => y + 1,
            };
            let en_passant = self.get_piece(x, target_y);
            if en_passant != Piece::WhitePawnMoved && en_passant != Piece::BlackPawnMoved {
                panic!(
                    "Can not capture piece, expected PawnMove, got {:?}",
                    en_passant
                );
            }
            self.set_piece((x, target_y), Piece::None);
        }
        self.move_piece((source_x, source_y), (x, y));
        self.try_promote_pawn((x, y), target);
        Ok(())
    }

    fn bishop_move_to(&mut self, target: &str) -> Result<()> {
        let (x, y) = BoardState::get_position(target);
        let (_, from_x, from_y) = self.find_piece(
            (x, y),
            &[(-1, -1), (1, -1), (1, 1), (-1, 1)],
            &[if self.current_player == CurrentPlayer::White {
                Piece::WhiteBishop
            } else {
                Piece::BlackBishop
            }],
        );
        self.move_piece((from_x, from_y), (x, y));
        Ok(())
    }

    fn knight_move_to(&mut self, target: &str) -> Result<()> {
        let mut offset = if &target[1..2] == "x" { 1 } else { 0 };
        let mut start_column = None;
        if target.as_bytes()[0] >= b'a'
            && target.as_bytes()[0] <= b'h'
            && target.as_bytes()[1 + offset] >= b'a'
            && target.as_bytes()[1 + offset] <= b'h'
        {
            let start = BoardState::get_position(&format!("{}1", target.as_bytes()[0] as char));
            start_column = Some(start.0);
            offset += 1;
        }
        let (x, y) = BoardState::get_position(&target[offset..]);
        let min_x = if x > 2 { x - 2 } else { 0 };
        let max_x = if x < 6 { x + 2 } else { 7 };
        let min_y = if y > 2 { y - 2 } else { 0 };
        let max_y = if y < 6 { y + 2 } else { 7 };

        for source_x in min_x..=max_x {
            for source_y in min_y..=max_y {
                if start_column.is_some() && start_column != Some(source_x) {
                    continue;
                }
                let piece = self.get_piece(source_x, source_y);
                if (piece == Piece::WhiteKnight && self.current_player == CurrentPlayer::White)
                    || (piece == Piece::BlackKnight && self.current_player == CurrentPlayer::Black)
                {
                    let delta_x = (x as i8 - source_x as i8).abs();
                    let delta_y = (y as i8 - source_y as i8).abs();
                    if (delta_x == 2 && delta_y == 1) || (delta_y == 2 && delta_x == 1) {
                        self.move_piece((source_x, source_y), (x, y));
                        return Ok(());
                    }
                }
            }
        }
        bail!("Could not find knight to move to {:?}", (x, y));
    }

    fn queen_move_to(&mut self, target: &str) -> Result<()> {
        let (x, y) = BoardState::get_position(target);
        let (_, from_x, from_y) = self.find_piece(
            (x, y),
            &[
                (-1, -1),
                (1, -1),
                (1, 1),
                (-1, 1),
                (-1, 0),
                (0, 1),
                (1, 0),
                (0, -1),
            ],
            &[if self.current_player == CurrentPlayer::White {
                Piece::WhiteQueen
            } else {
                Piece::BlackQueen
            }],
        );
        self.move_piece((from_x, from_y), (x, y));
        Ok(())
    }

    fn rook_move_to(&mut self, target: &str) -> Result<()> {
        let expected = if self.current_player == CurrentPlayer::White {
            &[Piece::WhiteRook, Piece::WhiteRookMoved]
        } else {
            &[Piece::BlackRook, Piece::BlackRookMoved]
        };
        let offset = if &target[1..2] == "x" { 1 } else { 0 };
        if target.as_bytes()[0] >= b'a'
            && target.as_bytes()[0] <= b'h'
            && target.as_bytes()[1 + offset] >= b'a'
            && target.as_bytes()[1 + offset] <= b'h'
        {
            let (source_x, _) = BoardState::get_position(&format!("{}1", &target[..1]));
            let (x, y) = BoardState::get_position(&target[offset + 1..]);
            if source_x == x {
                for source_y in 0..8 {
                    if source_y != y {
                        let piece = self.get_piece(source_x, source_y);
                        for e in expected {
                            if piece == *e {
                                self.move_piece((source_x, source_y), (x, y));
                                return Ok(());
                            }
                        }
                    }
                }
            } else {
                self.move_piece((source_x, y), (x, y));
                return Ok(());
            }
        }
        let (x, y) = BoardState::get_position(target);
        let (_, from_x, from_y) =
            self.find_piece((x, y), &[(-1, 0), (0, 1), (1, 0), (0, -1)], expected);
        self.move_piece((from_x, from_y), (x, y));
        Ok(())
    }

    fn king_move_to(&mut self, target: &str) -> Result<()> {
        let (x, y) = BoardState::get_position(target);
        let expected_piece = if self.current_player == CurrentPlayer::White {
            &[Piece::WhiteKing, Piece::WhiteKingMoved]
        } else {
            &[Piece::BlackKing, Piece::BlackKingMoved]
        };
        for delta_x in -1..=1 {
            for delta_y in -1..=1 {
                let new_x = delta_x + x as i8;
                let new_y = delta_y + y as i8;

                if new_x < 0 || new_x > 7 || new_y < 0 || new_y > 7 {
                    continue;
                }
                let piece = self.get_piece(new_x as u8, new_y as u8);
                if piece != Piece::None {
                    for expected in expected_piece {
                        if *expected == piece {
                            self.move_piece((new_x as u8, new_y as u8), (x, y));
                            return Ok(());
                        }
                    }
                }
            }
        }
        bail!("Could not find king to move")
    }

    fn castle_long(&mut self) -> Result<()> {
        match self.current_player {
            CurrentPlayer::White => {
                assert_eq!(Piece::WhiteKing, self.get_piece(3, 0));
                assert_eq!(Piece::WhiteRook, self.get_piece(7, 0));
                self.move_piece((3, 0), (5, 0));
                self.move_piece((7, 0), (4, 0));
                self.current_player = match self.current_player {
                    CurrentPlayer::White => CurrentPlayer::Black,
                    CurrentPlayer::Black => CurrentPlayer::White,
                };
            }
            CurrentPlayer::Black => {
                assert_eq!(Piece::BlackKing, self.get_piece(3, 7));
                assert_eq!(Piece::BlackRook, self.get_piece(7, 7));
                self.move_piece((3, 7), (5, 7));
                self.move_piece((7, 7), (4, 7));
                self.current_player = match self.current_player {
                    CurrentPlayer::White => CurrentPlayer::Black,
                    CurrentPlayer::Black => CurrentPlayer::White,
                };
            }
        }
        Ok(())
    }
    fn castle_short(&mut self) -> Result<()> {
        match self.current_player {
            CurrentPlayer::White => {
                assert_eq!(Piece::WhiteKing, self.get_piece(3, 0));
                assert_eq!(Piece::WhiteRook, self.get_piece(0, 0));
                self.move_piece((3, 0), (1, 0));
                self.move_piece((0, 0), (2, 0));
                self.current_player = match self.current_player {
                    CurrentPlayer::White => CurrentPlayer::Black,
                    CurrentPlayer::Black => CurrentPlayer::White,
                };
            }
            CurrentPlayer::Black => {
                assert_eq!(Piece::BlackKing, self.get_piece(3, 7));
                assert_eq!(Piece::BlackRook, self.get_piece(0, 7));
                self.move_piece((3, 7), (1, 7));
                self.move_piece((0, 7), (2, 7));
                self.current_player = match self.current_player {
                    CurrentPlayer::White => CurrentPlayer::Black,
                    CurrentPlayer::Black => CurrentPlayer::White,
                };
            }
        }
        Ok(())
    }

    pub fn make_move(&mut self, m: &str) -> Result<()> {
        println!("{:?} {:?}", self.current_player, m);
        if m.is_empty() {
            bail!("Empty move");
        }
        match m.as_bytes()[0] {
            b'Q' => self.queen_move_to(&m[1..]),
            b'N' => self.knight_move_to(&m[1..]),
            b'B' => self.bishop_move_to(&m[1..]),
            b'R' => self.rook_move_to(&m[1..]),
            b'K' => self.king_move_to(&m[1..]),
            _ => {
                if m == "O-O" {
                    self.castle_short()
                } else if m == "O-O-O" {
                    self.castle_long()
                } else if m.as_bytes().len() >= 4 && m.as_bytes()[1] == b'x' {
                    self.capture_with_pawn(&m[2..], &m[..1])
                } else {
                    self.move_pawn(m)
                }
            }
        }
    }

    pub fn to_piece_vec(&self) -> Vec<f32> {
        let mut result = Vec::with_capacity(Piece::BlackRookMoved as usize * 8 * 8);
        for i in Piece::WhiteKing as u8..=Piece::BlackRookMoved as u8 {
            let piece = Piece::from_u8(i).unwrap();
            for x in 0..8 {
                for y in 0..8 {
                    if self.pieces[y][x] == piece {
                        result.push(1.0f32);
                    } else {
                        result.push(0.0f32);
                    }
                }
            }
        }
        result
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub enum Piece {
        None,

        WhiteKing,
        WhiteKingMoved,
        WhiteQueen,
        WhiteBishop,
        WhiteKnight,
        WhitePawn,
        WhitePawnMoved,
        WhiteRook,
        WhiteRookMoved,

        BlackKing,
        BlackKingMoved,
        BlackQueen,
        BlackBishop,
        BlackKnight,
        BlackPawn,
        BlackPawnMoved,
        BlackRook,
        BlackRookMoved,
    }
}

impl Piece {
    pub fn has_moved(&mut self) {
        match self {
            Piece::WhiteKing => *self = Piece::WhiteKingMoved,
            Piece::WhitePawn => *self = Piece::WhitePawnMoved,
            Piece::WhiteRook => *self = Piece::WhiteRookMoved,

            Piece::BlackKing => *self = Piece::BlackKingMoved,
            Piece::BlackPawn => *self = Piece::BlackPawnMoved,
            Piece::BlackRook => *self = Piece::BlackRookMoved,
            _ => {}
        }
    }
}
