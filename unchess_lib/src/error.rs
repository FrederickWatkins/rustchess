use crate::types::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChessError {
    #[error("Piece not found at {0}")]
    PieceMissing(IntChessSquare),

    #[error("Illegal move attempted from {} to {}", 0.0, 0.1)]
    IllegalMove(ChessMove),

    #[error("Attempted to query piece of non-turn colour {0}")]
    WrongColour(IntChessSquare),

    #[error("Attempted to undo move when none have been played")]
    FirstMove,

    #[error("Move {0} impossible at current board state")]
    ImpossibleMove(AmbiguousMove),

    #[error("Underdefined move {0} with multiple solutions at current board state")]
    UnderdefinedMove(AmbiguousMove),

    #[error("Invalid FEN")]
    InvalidFEN,

    #[error("Invalid PGN, {0}")]
    InvalidPGN(String),

    #[error("Invalid position {0}")]
    InvalidPosition(String),

    #[error("Moves requested when none are available in current board state")]
    NoMoves,

    #[error("({0}, {1}) invalid square on chess board")]
    InvalidSquare(i8, i8),
}
