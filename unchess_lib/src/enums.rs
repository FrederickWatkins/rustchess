//! Module for enums common to all chess board representations

use std::ops::Not;

/// Colour of piece
#[allow(missing_docs)] // Enum variants self explanatory
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PieceColour {
    Black,
    White,
}

impl Not for PieceColour {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            PieceColour::Black => PieceColour::White,
            PieceColour::White => PieceColour::Black,
        }
    }
}

/// Type of piece
#[allow(missing_docs)] // Enum variants self explanatory
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PieceKind {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

impl From<PieceKind> for char {
    fn from(value: PieceKind) -> Self {
        match value {
            PieceKind::King => 'K',
            PieceKind::Queen => 'Q',
            PieceKind::Bishop => 'B',
            PieceKind::Knight => 'N',
            PieceKind::Rook => 'R',
            PieceKind::Pawn => 'P',
        }
    }
}

/// Basic states of board based on king safety
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoardState {
    /// Normal play in game, no restrictions on moves
    Normal,
    /// King is in check, only legal moves are ones that break the check
    Check,
    /// Game is over in a stalemate, king has no legal moves but not in check
    Stalemate,
    /// Game is over in a checkmate, king has no legal moves and is checked
    Checkmate,
}
