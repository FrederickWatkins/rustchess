//! Library wide error handling
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
/// Errors common across library interfaces
#[allow(missing_docs)] // Enum variants self documented by error messages
pub enum ChessError {
    #[error("Piece not found at {0:?}")]
    PieceNotFound(String),

    #[error("Board in invalid state")]
    InvalidBoard,

    #[error("Illegal move {0}")]
    IllegalMove(String),
}
