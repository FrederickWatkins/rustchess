//! Module for enums common to all chess board representations

/// Colour of piece
#[allow(missing_docs)] // Enum variants self explanatory
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PieceColour {
    Black,
    White,
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
