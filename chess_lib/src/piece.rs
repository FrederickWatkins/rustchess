use crate::types::Position;
use phf::{phf_map, Map};
use std::ops::Not;
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Colour {
    White,
    Black,
}

impl Not for Colour {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }
}

impl Colour {
    pub fn direction(self, direction: Position) -> Position {
        if self == Colour::Black {
            Position(direction.0, -direction.1)
        } else {
            Position(direction.0, direction.1)
        }
    }
}

pub static PIECE_LETTERS: Map<&'static str, PieceKind> = phf_map! {
    "P" => PieceKind::Pawn,
    "Kn" => PieceKind::Knight,
    "B" => PieceKind::Bishop,
    "R" => PieceKind::Rook,
    "Q" => PieceKind::Queen,
    "K" => PieceKind::King,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub enum PieceKind {
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
        PIECE_LETTERS
            .entries()
            .find(|(_key, value)| value == &&val)
            .unwrap()
            .0
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Piece {
    pub pos: Position,
    pub colour: Colour,
    pub kind: PieceKind,
}

impl Piece {
    pub fn new(pos: Position, colour: Colour, kind: PieceKind) -> Self {
        Piece { pos, colour, kind }
    }

    pub fn direction(&self, direction: Position) -> Position {
        self.colour.direction(direction)
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
        assert_eq!(
            <PieceKind as TryFrom<&str>>::try_from("Kn").unwrap(),
            PieceKind::Knight
        );
        assert_eq!(
            <PieceKind as TryFrom<&str>>::try_from("R").unwrap(),
            PieceKind::Rook
        );
        assert_eq!(
            <PieceKind as TryFrom<&str>>::try_from("B").unwrap(),
            PieceKind::Bishop
        );

        assert_eq!(<PieceKind as TryFrom<&str>>::try_from("G"), Err(()))
    }
}
