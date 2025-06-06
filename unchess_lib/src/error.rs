//! Library wide error handling
use std::fmt::Debug;
use thiserror::Error;

use crate::{
    enums::BoardState,
    simple_types::{SimpleMove, SimpleSquare},
};

#[derive(Error, Debug)]
/// Errors common across library interfaces
#[allow(missing_docs)] // Enum variants self documented by error messages
pub enum ChessError {
    #[error("Piece not found at {0}")]
    PieceNotFound(SimpleSquare),

    #[error("Board in invalid state, info: {0}")]
    InvalidBoard(String),

    #[error("Illegal move {0:?}")]
    IllegalMove(SimpleMove),

    #[error("File must be between 0-7 inclusive, {0} > 7")]
    InvalidFile(u8),

    #[error("Rank must be between 0-7 inclusive, {0} > 7")]
    InvalidRank(u8),

    #[error("{0:?} is not an actionable move")]
    NotAction(BoardState),

    #[error("Invalid PGN: {0}")]
    InvalidPGN(String),
}
