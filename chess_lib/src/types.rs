use crate::piece::{PieceKind, PIECE_LETTERS};
use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};
use phf::{Map, phf_map};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Position(pub i8, pub i8);

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        *self = rhs + *self;
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ChessMove(pub Position, pub Position);

static RANKS: Map<char, i8> = phf_map! {
    '1' => 0,
    '2' => 1,
    '3' => 2,
    '4' => 3,
    '5' => 4,
    '6' => 5,
    '7' => 6,
    '8' => 7,
};

static FILES: Map<char, i8> = phf_map! {
    'a' => 0,
    'b' => 1,
    'c' => 2,
    'd' => 3,
    'e' => 4,
    'f' => 5,
    'g' => 6,
    'h' => 7,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct AmbiguousMove {
    pub end: Position,
    pub kind: PieceKind,
    pub start_file: Option<i8>,
    pub start_rank: Option<i8>,
}

impl From<&str> for AmbiguousMove {
    fn from(value: &str) -> Self {
        let mut chars: Vec<char> = value.chars().filter(|ch| *ch != 'x' && *ch != '#' && *ch != '+').collect();
        let rank = RANKS.get(&chars.pop().unwrap()).unwrap();
        let file = FILES.get(&chars.pop().unwrap()).unwrap();
        let mut start_rank: Option<i8> = None;
        let mut start_file: Option<i8> = None;
        let mut kind = PieceKind::Pawn;
        for ch in chars.into_iter().rev() {
            if RANKS.contains_key(&ch) {
                start_rank = Some(*RANKS.get(&ch).unwrap())
            } else if FILES.contains_key(&ch) {
                start_file = Some(*FILES.get(&ch).unwrap())
            } else if PIECE_LETTERS.contains_key(&ch) {
                kind = PieceKind::try_from(ch).unwrap();
            }
        };
        Self { end: Position(*file, *rank), kind, start_file, start_rank }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pos_display() {
        assert_eq!(format!("{}", Position(5, 3)), *"(5, 3)");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(AmbiguousMove::from("e4"), AmbiguousMove{ end: Position(4, 3), kind: PieceKind::Pawn, start_file: None, start_rank: None });
        assert_eq!(AmbiguousMove::from("Bc7"), AmbiguousMove{ end: Position(2, 6), kind: PieceKind::Bishop, start_file: None, start_rank: None });
        assert_eq!(AmbiguousMove::from("Bb1"), AmbiguousMove{ end: Position(1, 0), kind: PieceKind::Bishop, start_file: None, start_rank: None });
        assert_eq!(AmbiguousMove::from("N7xc5"), AmbiguousMove{ end: Position(2, 4), kind: PieceKind::Knight, start_file: None, start_rank: Some(6) });
        assert_eq!(AmbiguousMove::from("Bb7xb6#"), AmbiguousMove{ end: Position(1, 5), kind: PieceKind::Bishop, start_file: Some(1), start_rank: Some(6) });
        assert_eq!(AmbiguousMove::from("Rh6xh4+"), AmbiguousMove{ end: Position(7, 3), kind: PieceKind::Rook, start_file: Some(7), start_rank: Some(5) });
        assert_eq!(AmbiguousMove::from("Nge7+"), AmbiguousMove{ end: Position(4, 6), kind: PieceKind::Knight, start_file: Some(6), start_rank: None });
        assert_eq!(AmbiguousMove::from("h6"), AmbiguousMove{ end: Position(7, 5), kind: PieceKind::Pawn, start_file: None, start_rank: None });
        assert_eq!(AmbiguousMove::from("h7"), AmbiguousMove{ end: Position(7, 6), kind: PieceKind::Pawn, start_file: None, start_rank: None });
    }
}