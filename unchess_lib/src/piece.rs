use crate::types::IntChessSquare;
use phf::{phf_map, Map};
use strum::{EnumCount, FromRepr};
use std::ops::Not;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, FromRepr)]
pub enum Colour {
    White = 1,
    Black = 0,
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
    pub fn direction(self, direction: IntChessSquare) -> IntChessSquare {
        if self == Colour::Black {
            IntChessSquare(direction.0, -direction.1)
        } else {
            IntChessSquare(direction.0, direction.1)
        }
    }

    pub fn back_rank(self) -> i8 {
        match self {
            Colour::White => 0,
            Colour::Black => 7,
        }
    }
}

pub static PIECE_LETTERS: Map<char, PieceKind> = phf_map! {
    'P' => PieceKind::Pawn,
    'N' => PieceKind::Knight,
    'B' => PieceKind::Bishop,
    'R' => PieceKind::Rook,
    'Q' => PieceKind::Queen,
    'K' => PieceKind::King,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, EnumCount, FromRepr)]
pub enum PieceKind {
    Knight,
    Pawn,
    Bishop,
    Rook,
    Queen,
    King,
}

impl TryFrom<char> for PieceKind {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if let Some(x) = PIECE_LETTERS.get(&value) {
            Ok(*x)
        } else {
            Err(())
        }
    }
}

impl From<PieceKind> for char {
    fn from(val: PieceKind) -> Self {
        *PIECE_LETTERS
            .entries()
            .find(|(_key, value)| value == &&val)
            .unwrap()
            .0
    }
}

impl PieceKind {
    pub fn value(&self) -> u64 {
        match self {
            PieceKind::Pawn => 1,
            PieceKind::Knight => 3,
            PieceKind::Bishop => 3,
            PieceKind::Rook => 5,
            PieceKind::Queen => 9,
            PieceKind::King => 0,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Piece {
    pub pos: IntChessSquare,
    pub colour: Colour,
    pub kind: PieceKind,
}

impl From<Piece> for char {
    fn from(value: Piece) -> Self {
        if value.colour == Colour::White {
            char::from(value.kind).to_ascii_uppercase()
        } else {
            char::from(value.kind).to_ascii_lowercase()
        }
    }
}

impl Piece {
    pub fn new(pos: IntChessSquare, colour: Colour, kind: PieceKind) -> Self {
        Piece { pos, colour, kind }
    }

    pub fn direction(&self, direction: IntChessSquare) -> IntChessSquare {
        self.colour.direction(direction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let pos1 = IntChessSquare(3, 5);
        let pos2 = IntChessSquare(4, 3);
        let result = pos1 + pos2;
        assert_eq!(result, IntChessSquare(7, 8))
    }

    #[test]
    fn test_piece_letters() {
        assert_eq!(PIECE_LETTERS.get(&'K').unwrap(), &PieceKind::King);
        assert_eq!(PIECE_LETTERS.get(&'N').unwrap(), &PieceKind::Knight);
        assert_eq!(PIECE_LETTERS.get(&'R').unwrap(), &PieceKind::Rook);

        assert_eq!(
            PIECE_LETTERS
                .entries()
                .find(|(_key, value)| value == &&PieceKind::Pawn)
                .unwrap()
                .0,
            &'P'
        );
        assert_eq!(
            PIECE_LETTERS
                .entries()
                .find(|(_key, value)| value == &&PieceKind::Queen)
                .unwrap()
                .0,
            &'Q'
        );
        assert_eq!(
            PIECE_LETTERS
                .entries()
                .find(|(_key, value)| value == &&PieceKind::Bishop)
                .unwrap()
                .0,
            &'B'
        );
    }

    #[test]
    fn test_from_piecekind() {
        assert_eq!(<char as From<PieceKind>>::from(PieceKind::Pawn), 'P');
        assert_eq!(<char as From<PieceKind>>::from(PieceKind::King), 'K');
        assert_eq!(<char as From<PieceKind>>::from(PieceKind::Queen), 'Q');
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            <PieceKind as TryFrom<char>>::try_from('N').unwrap(),
            PieceKind::Knight
        );
        assert_eq!(
            <PieceKind as TryFrom<char>>::try_from('R').unwrap(),
            PieceKind::Rook
        );
        assert_eq!(
            <PieceKind as TryFrom<char>>::try_from('B').unwrap(),
            PieceKind::Bishop
        );

        assert_eq!(<PieceKind as TryFrom<char>>::try_from('G'), Err(()))
    }
}
