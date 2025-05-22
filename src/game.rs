use phf::{phf_map, Map};
use std::{fmt::Display, ops::Add};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Position(i64, i64);

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Colour {
    Black,
    White,
}

impl Colour {
    pub fn direction(self) -> Position {
        if self == Colour::Black {
            Position(0, -1)
        } else {
            Position(0, 1)
        }
    }
}

static PIECE_LETTERS: Map<&'static str, PieceKind> = phf_map! {
    "P" => PieceKind::Pawn,
    "Kn" => PieceKind::Knight,
    "B" => PieceKind::Bishop,
    "R" => PieceKind::Rook,
    "Q" => PieceKind::Queen,
    "K" => PieceKind::King,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl TryFrom<&str> for PieceKind {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(x) = PIECE_LETTERS.get(value) {
            Ok(*x)
        } else {
            Err(())
        }
    }
}

impl From<PieceKind> for &str {
    fn from(val: PieceKind) -> Self {
        PIECE_LETTERS.entries().find(|(_key, value)| value == &&val).unwrap().0
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PieceBehaviour {
    Pawn { moved: bool },
    Knight,
    Bishop,
    Rook { moved: bool },
    Queen,
    King { moved: bool },
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Piece {
    pos: Position,
    colour: Colour,
    behaviour: PieceBehaviour,
}

impl Piece {
    fn new(pos: Position, colour: Colour, behaviour: PieceBehaviour) -> Self {
        Piece { pos, colour, behaviour }
    }

    fn direction(&self) -> Position {
        self.colour.direction()
    }
}

struct Move {
    end: Position,
    start: Option<Position>,
    kind: Option<PieceKind>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Board {
    pieces: Vec<Piece>,
    turn: Colour,
    // The square that the en-passanting pawn can move to as used in FEN
    en_passant: Option<Position>,
}

impl Board {
    fn get_all_pieces(&self) -> Vec<&Piece> {
        self.pieces.iter().collect()
    }

    fn get_piece(&self, pos: Position) -> Option<&Piece> {
        self.pieces.iter().find(|&piece| piece.pos == pos)
    }

    fn get_piece_mut(&mut self, pos: Position) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|piece| piece.pos == pos)
    }

    fn check_move(&self, start: Position, end: Position) -> Result<bool, ()> {
        Ok(self.get_piece_moves(start)?.contains(&end))
    }

    fn get_all_moves(&self) -> Vec<(Position, Position)> {
        todo!()
    }

    fn get_piece_moves(&self, pos: Position) -> Result<Vec<Position>, ()> {
        if let Some(piece) = self.get_piece(pos) {
            match piece.behaviour {
                PieceBehaviour::Pawn { .. } => Ok(self.pawn_moves(piece)),
                PieceBehaviour::Knight => Ok(self.knight_moves(piece)),
                PieceBehaviour::Bishop => Ok(self.bishop_moves(piece)),
                PieceBehaviour::Rook { .. } => Ok(self.rook_moves(piece)),
                PieceBehaviour::Queen => Ok(self.queen_moves(piece)),
                PieceBehaviour::King { .. } => Ok(self.king_moves(piece)),
            }
        } else {
            Err(())
        }
    }

    fn move_piece(&mut self, start: Position, end: Position) -> Result<(), ()> {
        if self.check_move(start, end)? {
            self.move_piece_unchecked(start, end);
            Ok(())
        } else {
            Err(())
        }
    }

    fn move_piece_unchecked(&mut self, start: Position, end: Position) {
        if let Some(piece) = self.get_piece_mut(start) {
            piece.pos = end
        }
    }

    fn pawn_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn knight_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn bishop_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn rook_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn queen_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn king_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn fmt_board(&self) -> String {
        let templ = "Xx Xx Xx Xx Xx Xx Xx Xx\n";
        let mut outstr = String::from(templ).repeat(8);
        for piece in &self.pieces {
            let index = (7 - piece.pos.1 as usize) * templ.len() + piece.pos.0 as usize * 3;
            outstr.replace_range(
                index..index + 2,
                match piece.behaviour {
                    PieceBehaviour::Pawn { .. } => "P ",
                    PieceBehaviour::Knight => "Kn",
                    PieceBehaviour::Bishop => "B ",
                    PieceBehaviour::Rook { .. } => "R ",
                    PieceBehaviour::Queen => "Q ",
                    PieceBehaviour::King { .. } => "K ",
                },
            );
        }
        outstr
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.fmt_board())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let pos1 = Position(3, 5);
        let pos2 = Position(4, 3);
        let result = pos1 + pos2;
        assert_eq!(result, Position(7, 8))
    }

    #[test]
    fn test_piece_letters() {
        assert_eq!(PIECE_LETTERS.get("K").unwrap(), &PieceKind::King);
        assert_eq!(PIECE_LETTERS.get("Kn").unwrap(), &PieceKind::Knight);
        assert_eq!(PIECE_LETTERS.get("R").unwrap(), &PieceKind::Rook);

        assert_eq!(
            PIECE_LETTERS
                .entries()
                .find(|(_key, value)| value == &&PieceKind::Pawn)
                .unwrap()
                .0,
            &"P"
        );
        assert_eq!(
            PIECE_LETTERS
                .entries()
                .find(|(_key, value)| value == &&PieceKind::Queen)
                .unwrap()
                .0,
            &"Q"
        );
        assert_eq!(
            PIECE_LETTERS
                .entries()
                .find(|(_key, value)| value == &&PieceKind::Bishop)
                .unwrap()
                .0,
            &"B"
        );
    }
    
    #[test]
    fn test_from_piecekind() {
        assert_eq!(<&str as From<PieceKind>>::from(PieceKind::Pawn), "P");
        assert_eq!(<&str as From<PieceKind>>::from(PieceKind::King), "K");
        assert_eq!(<&str as From<PieceKind>>::from(PieceKind::Queen), "Q");
    }
    
    #[test]
    fn test_from_str() {
        assert_eq!(<PieceKind as TryFrom<&str>>::try_from("Kn").unwrap(), PieceKind::Knight);
        assert_eq!(<PieceKind as TryFrom<&str>>::try_from("R").unwrap(), PieceKind::Rook);
        assert_eq!(<PieceKind as TryFrom<&str>>::try_from("B").unwrap(), PieceKind::Bishop);
        
        assert_eq!(<PieceKind as TryFrom<&str>>::try_from("G"), Err(()))
    }
}
