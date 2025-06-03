//! Library wide error handling
use std::fmt::Debug;
use thiserror::Error;

use crate::traits::{ChessMove, ChessSquare};

#[derive(Error, Debug)]
/// Errors common across library interfaces
#[allow(missing_docs)] // Enum variants self documented by error messages
pub enum ChessError {
    #[error("Piece not found at {0}")]
    PieceNotFound(Box<dyn ChessSquare>),

    #[error("Board in invalid state, info: {0}")]
    InvalidBoard(String),

    #[error("Illegal move {0}")]
    IllegalMove(Box<dyn ChessMove<dyn ChessSquare>>),

    #[error("File must be between 0-7 inclusive, {0} > 7")]
    InvalidFile(u8),

    #[error("Rank must be between 0-7 inclusive, {0} > 7")]
    InvalidRank(u8),
}
