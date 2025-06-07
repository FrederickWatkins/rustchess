//! Default types used across crate.
//!
//! These types are transparent representations, compared to the more complex internals of the
//! bittwiddling versions, so they are used for error types and such.
use core::fmt;

#[cfg(test)]
use proptest::prelude::Strategy;

use crate::enums::PieceColour;
use crate::enums::PieceKind;
use crate::error::ChessError;
use crate::parser::pgn;
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

    /// Create square from PGN standard string
    ///
    /// # Errors
    /// [`crate::error::ChessError::InvalidPGN`] if input is invalid
    pub fn from_pgn_str(input: &str) -> Result<Self, ChessError> {
        if let Ok(s) = pgn::square(input) {
            Ok(s.1)
        } else {
            Err(ChessError::InvalidPGN(input.to_string()))
        }
    }

    /// Strategy for valid squares
    #[cfg(test)]
    pub fn strategy() -> impl Strategy<Value = Self> {
        use proptest::prelude::any;

        let file = any::<u8>().prop_filter("Valid range for file", |x| (0..=7).contains(x));
        let rank = any::<u8>().prop_filter("Valid range for rank", |x| (0..=7).contains(x));
        (file, rank).prop_map(|(file, rank)| Self::new(file, rank))
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

    /// Create move from PGNish string, but unambiguous
    ///
    /// # Errors
    /// [crate::error::ChessError::InvalidPGN] if `pgn` is invalid
    pub fn from_pgn_str(pgn: &str) -> Result<Self, ChessError> {
        if let Ok(m) = pgn::unambiguous_move(pgn) {
            Ok(m.1)
        } else {
            Err(ChessError::InvalidPGN(pgn.to_string()))
        }
    }

    /// Strategy for property testing moves
    ///
    /// NOTE: to avoid generating invalid moves to and from the same square, if they are generated
    /// it replaces them with the move a1h8
    #[cfg(test)]
    pub fn strategy() -> impl Strategy<Value = Self> {
        use proptest::option::of;

        let src = SimpleSquare::strategy();
        let dest = SimpleSquare::strategy();
        let promote_to = of(PieceKind::promotable_stategy());
        (src, dest, promote_to).prop_map(|(src, dest, promote_to)| {
            if src != dest {
                Self::new(src, dest, promote_to)
            } else {
                Self::from_pgn_str("a1h8").unwrap()
            }
        })
    }
}

/// Simple minimum piece type
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl SimplePiece {
    /// New piece
    pub fn new(kind: PieceKind, colour: PieceColour) -> Self {
        Self { kind, colour }
    }

    /// Strategy for any piece of any colour
    #[cfg(test)]
    pub fn strategy() -> impl Strategy<Value = Self> {
        let kind = PieceKind::strategy();
        let colour = PieceColour::strategy();

        (kind, colour).prop_map(|(kind, colour)| Self::new(kind, colour))
    }
}

impl From<SimplePiece> for char {
    /// As FEN standard char (White is uppercase, black is lowercase)
    fn from(value: SimplePiece) -> Self {
        let kind = char::from(value.kind);
        match value.colour {
            PieceColour::Black => kind.to_ascii_lowercase(),
            PieceColour::White => kind.to_ascii_uppercase(),
        }
    }
}
