//! Module for game notations like PGN and FEN

use crate::{enums::AmbiguousMove, error::ChessError, parser};

/// Convert u8 representation of file into char based on pgn standard
///
/// # Errors
/// - [`crate::error::ChessError::InvalidFile`] if `file` not between 0-7 inclusive
pub fn file_to_char(file: u8) -> Result<char, ChessError> {
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
pub fn rank_to_char(rank: u8) -> Result<char, ChessError> {
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

/// Convert u8 representation of file into char based on pgn standard
///
/// # Errors
/// - [`crate::error::ChessError::InvalidFile`] if `file` not between a-h inclusive
pub fn char_to_file(file: char) -> Result<u8, ChessError> {
    match file {
        'a' => Ok(0),
        'b' => Ok(1),
        'c' => Ok(2),
        'd' => Ok(3),
        'e' => Ok(4),
        'f' => Ok(5),
        'g' => Ok(6),
        'h' => Ok(7),
        _ => Err(ChessError::InvalidFile(file as u8)),
    }
}

/// Convert char representation of rank into u8 based on pgn standard
///
/// # Errors
/// - [`crate::error::ChessError::InvalidRank`] if `rank` not between 1-8 inclusive
pub fn char_to_rank(rank: char) -> Result<u8, ChessError> {
    match rank {
        '1' => Ok(0),
        '2' => Ok(1),
        '3' => Ok(2),
        '4' => Ok(3),
        '5' => Ok(4),
        '6' => Ok(5),
        '7' => Ok(6),
        '8' => Ok(7),
        _ => Err(ChessError::InvalidRank(rank as u8)),
    }
}

/// Convert pgn file to vector of moves
///
/// # Errors
/// - [`crate::error::ChessError::InvalidPGN`] if PGN can't be parsed
pub fn pgn_to_moves(input: &str) -> Result<Vec<AmbiguousMove>, ChessError> {
    if let Ok((_, (_, moves))) = parser::pgn::pgn(input) {
        Ok(moves)
    } else {
        Err(ChessError::InvalidPGN(input.to_string()))
    }
}
