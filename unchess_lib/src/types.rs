use crate::{
    error::ChessError,
    piece::{PieceKind, PIECE_LETTERS},
};
use phf::{phf_map, Map};
use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

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

impl TryFrom<&str> for Position {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 2 {
            Ok(Self(
                file_to_i8(value.chars().next().unwrap()),
                rank_to_i8(value.chars().next_back().unwrap()),
            ))
        } else {
            Err(ChessError::InvalidPosition(String::from(value)))
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", i8_to_file(self.0), i8_to_rank(self.1))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum CastlingSide {
    QueenSide,
    KingSide,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ChessMove {
    pub start: Position,
    pub end: Position,
    pub promote: Option<PieceKind>,
}

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

pub fn rank_to_i8(ch: char) -> i8 {
    *RANKS.get(&ch).unwrap()
}

pub fn i8_to_rank(x: i8) -> char {
    *RANKS
        .entries()
        .find(|(_key, value)| value == &&x)
        .unwrap()
        .0
}

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

pub fn file_to_i8(ch: char) -> i8 {
    *FILES.get(&ch).unwrap()
}

pub fn i8_to_file(x: i8) -> char {
    *FILES
        .entries()
        .find(|(_key, value)| value == &&x)
        .unwrap()
        .0
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum AmbiguousMove {
    Standard {
        end: Position,
        kind: PieceKind,
        start_file: Option<i8>,
        start_rank: Option<i8>,
        promote: Option<PieceKind>,
    },
    Castle(CastlingSide),
}

impl Display for AmbiguousMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AmbiguousMove::Standard {
                end,
                kind,
                start_file,
                start_rank,
                promote,
            } => {
                match char::from(*kind) {
                    'P' => (),
                    ch => write!(f, "{}", ch)?,
                }
                if let Some(file) = start_file {
                    write!(f, "{}", i8_to_file(*file))?;
                }
                if let Some(rank) = start_rank {
                    write!(f, "{}", i8_to_rank(*rank))?;
                }
                write!(f, "{}", end)?;
                if let Some(pro) = promote {
                    write!(f, "{}", '=')?;
                    write!(f, "{}", char::from(*pro))
                } else {
                    Ok(())
                }
            }
            AmbiguousMove::Castle(castling_side) => match castling_side {
                CastlingSide::QueenSide => write!(f, "O-O-O"),
                CastlingSide::KingSide => write!(f, "O-O"),
            },
        }
    }
}

impl TryFrom<&str> for AmbiguousMove {
    type Error = ChessError;

    fn try_from(value: &str) -> Result<AmbiguousMove, ChessError> {
        if value == "O-O-O" {
            return Ok(Self::Castle(CastlingSide::QueenSide));
        }
        if value == "O-O" {
            return Ok(Self::Castle(CastlingSide::KingSide));
        }
        let mut chars: Vec<char> = value
            .split('=')
            .nth(0)
            .ok_or(ChessError::InvalidPGN(String::from(value)))?
            .chars()
            .filter(|ch| *ch != 'x' && *ch != '#' && *ch != '+')
            .collect();
        let rank = RANKS
            .get(
                &chars
                    .pop()
                    .ok_or(ChessError::InvalidPGN(String::from(value)))?,
            )
            .ok_or(ChessError::InvalidPGN(String::from(value)))?;
        let file = FILES
            .get(
                &chars
                    .pop()
                    .ok_or(ChessError::InvalidPGN(String::from(value)))?,
            )
            .ok_or(ChessError::InvalidPGN(String::from(value)))?;
        let mut start_rank: Option<i8> = None;
        let mut start_file: Option<i8> = None;
        let mut kind = PieceKind::Pawn;
        for ch in chars.into_iter().rev() {
            if RANKS.contains_key(&ch) {
                start_rank = Some(
                    *RANKS
                        .get(&ch)
                        .ok_or(ChessError::InvalidPGN(String::from(value)))?,
                )
            } else if FILES.contains_key(&ch) {
                start_file = Some(
                    *FILES
                        .get(&ch)
                        .ok_or(ChessError::InvalidPGN(String::from(value)))?,
                )
            } else if PIECE_LETTERS.contains_key(&ch) {
                kind = PieceKind::try_from(ch)
                    .map_err(|()| ChessError::InvalidPGN(String::from(value)))?;
            }
        }
        Ok(Self::Standard {
            end: Position(*file, *rank),
            kind,
            start_file,
            start_rank,
            promote: if let Some(x) = value.split('=').nth(1) {
                Some(
                    PieceKind::try_from(
                        x.chars()
                            .next()
                            .ok_or(ChessError::InvalidPGN(String::from(value)))?,
                    )
                    .map_err(|()| ChessError::InvalidPGN(String::from(value)))?,
                )
            } else {
                None
            },
        })
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum BoardState {
    Normal,
    Check,
    Checkmate,
    Stalemate,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pos_display() {
        assert_eq!(format!("{}", Position(5, 3)), *"f4");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            AmbiguousMove::try_from("e4").unwrap(),
            AmbiguousMove::Standard {
                end: Position(4, 3),
                kind: PieceKind::Pawn,
                start_file: None,
                start_rank: None,
                promote: None,
            }
        );
        assert_eq!(
            AmbiguousMove::try_from("Bc7").unwrap(),
            AmbiguousMove::Standard {
                end: Position(2, 6),
                kind: PieceKind::Bishop,
                start_file: None,
                start_rank: None,
                promote: None,
            }
        );
        assert_eq!(
            AmbiguousMove::try_from("Bb1").unwrap(),
            AmbiguousMove::Standard {
                end: Position(1, 0),
                kind: PieceKind::Bishop,
                start_file: None,
                start_rank: None,
                promote: None,
            }
        );
        assert_eq!(
            AmbiguousMove::try_from("N7xc5").unwrap(),
            AmbiguousMove::Standard {
                end: Position(2, 4),
                kind: PieceKind::Knight,
                start_file: None,
                start_rank: Some(6),
                promote: None,
            }
        );
        assert_eq!(
            AmbiguousMove::try_from("Bb7xb6#").unwrap(),
            AmbiguousMove::Standard {
                end: Position(1, 5),
                kind: PieceKind::Bishop,
                start_file: Some(1),
                start_rank: Some(6),
                promote: None,
            }
        );
        assert_eq!(
            AmbiguousMove::try_from("Rh6xh4+").unwrap(),
            AmbiguousMove::Standard {
                end: Position(7, 3),
                kind: PieceKind::Rook,
                start_file: Some(7),
                start_rank: Some(5),
                promote: None,
            }
        );
        assert_eq!(
            AmbiguousMove::try_from("Nge7+").unwrap(),
            AmbiguousMove::Standard {
                end: Position(4, 6),
                kind: PieceKind::Knight,
                start_file: Some(6),
                start_rank: None,
                promote: None,
            }
        );
        assert_eq!(
            AmbiguousMove::try_from("h6").unwrap(),
            AmbiguousMove::Standard {
                end: Position(7, 5),
                kind: PieceKind::Pawn,
                start_file: None,
                start_rank: None,
                promote: None,
            }
        );
        assert_eq!(
            AmbiguousMove::try_from("h8=Q").unwrap(),
            AmbiguousMove::Standard {
                end: Position(7, 7),
                kind: PieceKind::Pawn,
                start_file: None,
                start_rank: None,
                promote: Some(PieceKind::Queen),
            }
        );
    }
}
