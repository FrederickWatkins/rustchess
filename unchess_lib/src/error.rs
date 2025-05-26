use crate::types::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChessError {
    #[error("Piece not found at {0}")]
    PieceMissing(Position),

    #[error("Illegal move attempted from {} to {}", 0.0, 0.1)]
    IllegalMove(ChessMove),

    #[error("Attempted to query piece of non-turn colour {0}")]
    WrongColour(Position),

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
}
