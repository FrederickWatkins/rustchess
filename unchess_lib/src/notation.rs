//! Module for game notations like PGN and FEN

use crate::{enums::AmbiguousMove, error::ChessError, pgn};

/// Convert u8 representation of file into char based on pgn standard
///
/// # Errors
/// - [`crate::error::ChessError::InvalidFile`] if `file` not between 0-7 inclusive
pub fn file(file: u8) -> Result<char, ChessError> {
    match file {
        0 => Ok('a'),
        1 => Ok('b'),
        2 => Ok('c'),
        3 => Ok('d'),
        4 => Ok('e'),
        5 => Ok('f'),
        6 => Ok('g'),
        7 => Ok('h'),
        _ => Err(ChessError::InvalidFile(file)),
    }
}

/// Convert u8 representation of rank into char based on pgn standard
///
/// # Errors
/// - [`crate::error::ChessError::InvalidRank`] if `rank` not between 0-7 inclusive
pub fn rank(rank: u8) -> Result<char, ChessError> {
    match rank {
        0 => Ok('1'),
        1 => Ok('2'),
        2 => Ok('3'),
        3 => Ok('4'),
        4 => Ok('5'),
        5 => Ok('6'),
        6 => Ok('7'),
        7 => Ok('8'),
        _ => Err(ChessError::InvalidRank(rank)),
    }
}

/// Convert pgn file to vector of moves
/// 
/// # Errors
/// - [`crate::error::ChessError::InvalidPGN`] if PGN can't be parsed
pub fn pgn_to_moves(input: &str) -> Result<Vec<AmbiguousMove>, ChessError> {
    if let Ok((_, (_, moves))) =  pgn::pgn(input) {
        Ok(moves)
    } else {
        Err(ChessError::InvalidPGN(input.to_string()))
    }
}
