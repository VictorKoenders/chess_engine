use shared::{BoardState, Result};

#[derive(Default, Debug, Copy, Clone)]
pub struct Notation {
    pub piece: Piece,
    pub from_row: Option<Row>,
    pub from_col: Option<Col>,
    pub row: Row,
    pub col: Col,
    pub is_capturing_piece: bool,
    pub promote_piece: Option<Piece>,
    pub is_check: bool,
    pub is_checkmate: bool,
    pub is_short_castle: bool,
    pub is_long_castle: bool,
}

impl Notation {
    fn long_castle() -> Self {
        Notation {
            is_long_castle: true,
            ..Notation::default()
        }
    }
    fn short_castle() -> Self {
        Notation {
            is_short_castle: true,
            ..Notation::default()
        }
    }
}

pub type Row = u8;
pub type Col = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl Default for Piece {
    fn default() -> Piece {
        Piece::Pawn
    }
}

impl Notation {
    pub fn parse(s: &str) -> Result<Notation> {
        if s.starts_with("O-O-O") {
            return Ok(Notation::long_castle());
        }
        if s.starts_with("O-O") {
            return Ok(Notation::short_castle());
        }
        let mut notation = Notation::default();
        let mut bytes = s.as_bytes().iter().peekable();

        match bytes.peek() {
            Some(b'Q') => notation.piece = Piece::Queen,
            Some(b'R') => notation.piece = Piece::Rook,
            Some(b'B') => notation.piece = Piece::Bishop,
            Some(b'K') => notation.piece = Piece::King,
            Some(b'N') => notation.piece = Piece::Knight,
            _ => {}
        };
        if notation.piece != Piece::Pawn {
            // consume the peeked byte
            bytes.next();
        }

        let mut b = *bytes.next().ok_or_else(|| format_err!("Unexpected end"))?;

        if b >= b'a' && b <= b'z' {
            let after = **bytes.peek().unwrap_or(&&b' ');
            if after == b'x' || (after >= b'a' && after <= b'h') {
                notation.from_row = Some(b - b'1');
                b = *bytes.next().ok_or_else(|| format_err!("Unexpected end"))?;
            }
        }
        if b >= b'1' && b <= b'8' {
            notation.from_col = Some(b - b'1');
            b = *bytes.next().ok_or_else(|| format_err!("Unexpected end"))?;
        }
        if b == b'x' {
            notation.is_capturing_piece = true;
            b = *bytes.next().ok_or_else(|| format_err!("Unexpected end"))?;
        }

        assert!(b >= b'a' && b <= b'z');
        notation.col = b - b'a';
        b = *bytes.next().ok_or_else(|| format_err!("Unexpected end"))?;
        assert!(b >= b'1' && b <= b'8');
        notation.row = b - b'1';

        if let Some(&&b'x') = bytes.peek() {
            notation.is_capturing_piece = true;
            bytes.next();
        }
        if let Some(&&c) = bytes.peek() {
            if c >= b'a' && c <= b'h' {
                // we have found a 2nd location
                let col = *bytes
                    .next()
                    .ok_or_else(|| format_err!("Could not read 2nd position"))?;
                let row = *bytes
                    .next()
                    .ok_or_else(|| format_err!("Could not read 2nd position"))?;
                notation.from_row = Some(notation.row);
                notation.from_col = Some(notation.col);
                notation.col = col - b'a';
                notation.row = row - b'1';
            }
        }

        if let Some(&&b'=') = bytes.peek() {
            bytes.next();
            let piece = bytes
                .next()
                .ok_or_else(|| format_err!("Expected promoted piece, got end"))?;
            notation.promote_piece = match piece {
                b'Q' => Some(Piece::Queen),
                b'N' => Some(Piece::Knight),
                b'R' => Some(Piece::Rook),
                b'B' => Some(Piece::Bishop),
                x => panic!("Unknown piece promotion: {:?}", x),
            };
        }
        if let Some(&&b'+') = bytes.peek() {
            notation.is_check = true;
            bytes.next();
        }
        if let Some(&&b'#') = bytes.peek() {
            notation.is_checkmate = true;
            bytes.next();
        }
        println!("Remaining: {:?}", bytes.peek());
        assert!(bytes.next().is_none());
        Ok(notation)
    }

    pub fn apply(&self, _boardstate: &mut BoardState) -> Result<()> {
        bail!("Not implemented")
    }
}

#[test]
fn test_command_knight_move() {
    let notation = Notation::parse("Na7b8+").expect("Could not parse \"Ng7f5+\"");
    assert_eq!(Piece::Knight, notation.piece);
    assert_eq!(Some(6), notation.from_row);
    assert_eq!(Some(0), notation.from_col);
    assert_eq!(7, notation.row);
    assert_eq!(1, notation.col);
    assert_eq!(false, notation.is_capturing_piece);
    assert_eq!(None, notation.promote_piece);
    assert_eq!(true, notation.is_check);
    assert_eq!(false, notation.is_checkmate);
    assert_eq!(false, notation.is_short_castle);
    assert_eq!(false, notation.is_long_castle);
}

#[test]
fn test_command_rook_move() {
    let notation = Notation::parse("Rd7xd2+").expect("Could not parse \"Rd7xd2+\"");
    assert_eq!(Piece::Rook, notation.piece);
    assert_eq!(Some(6), notation.from_row);
    assert_eq!(Some(3), notation.from_col);
    assert_eq!(1, notation.row);
    assert_eq!(3, notation.col);
    assert_eq!(true, notation.is_capturing_piece);
    assert_eq!(None, notation.promote_piece);
    assert_eq!(true, notation.is_check);
    assert_eq!(false, notation.is_checkmate);
    assert_eq!(false, notation.is_short_castle);
    assert_eq!(false, notation.is_long_castle);
}
