//! Default types used across crate.
//!
//! These types are transparent representations, compared to the more complex internals of the
//! bittwiddling versions, so they are used for error types and such.
use core::fmt;

use crate::enums::PieceColour;
use crate::enums::PieceKind;
use crate::traits::ChessMove;
use crate::traits::ChessPiece;
use crate::traits::ChessSquare;

/// Chess square
///
/// Internally represented as two u8s, for file and rank. Used by error types and non bittwiddling
/// boards.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleSquare {
    file: u8,
    rank: u8,
}

impl ChessSquare for SimpleSquare {
    fn file(&self) -> u8 {
        self.file
    }

    fn rank(&self) -> u8 {
        self.rank
    }
}

impl fmt::Display for SimpleSquare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl SimpleSquare {
    /// Chess square at `file` and `rank`
    ///
    /// # Panics
    /// Panics if file and/or rank are not between 0-7 inclusive
    pub fn new(file: u8, rank: u8) -> Self {
        assert!((0..8).contains(&file), "File must be between 0-7 inclusive, {file} > 7");
        assert!((0..8).contains(&rank), "Rank must be between 0-7 inclusive, {rank} > 7");
        Self { file, rank }
    }
}

/// Chess move from src to dest
///
/// Internally uses [`SimpleSquare`] so used for error types and piece lists.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleMove {
    src: SimpleSquare,
    dest: SimpleSquare,
    promote_to: Option<PieceKind>,
}

impl ChessMove<SimpleSquare> for SimpleMove {
    fn src(&self) -> SimpleSquare {
        self.src
    }

    fn dest(&self) -> SimpleSquare {
        self.dest
    }

    fn promote_to(&self) -> Option<PieceKind> {
        self.promote_to
    }
}

impl fmt::Display for SimpleMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl SimpleMove {
    /// Chess move from `src` to `dest`
    ///
    /// # Panics
    /// Panics if source and destination are the same square
    pub fn new(src: SimpleSquare, dest: SimpleSquare, promote_to: Option<PieceKind>) -> Self {
        assert_ne!(src, dest, "Chess move cannot originate and terminate at same square");
        Self { src, dest, promote_to }
    }
}

/// Simple minimum piece type
pub struct SimplePiece {
    kind: PieceKind,
    colour: PieceColour,
}

impl ChessPiece for SimplePiece {
    fn kind(&self) -> PieceKind {
        self.kind
    }

    fn colour(&self) -> PieceColour {
        self.colour
    }
}
