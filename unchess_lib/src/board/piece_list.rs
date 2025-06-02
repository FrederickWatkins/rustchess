//! Piece list representation of a chess board
//!
//! Uses vectors of pieces to represent the chess board. This implementation is far simpler and
//! doesn't involve bit-twiddling, so is less likely to contain bugs, but is very memory heavy and
//! slow.

use std::fmt;

use crate::enums::PieceColour;
use crate::enums::PieceKind;
use crate::error::ChessError;
use crate::notation;
use crate::traits;

use tracing::{Level, event};

/// Chess square
///
/// Internally represented as two u8s, for file and rank.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChessSquare {
    file: u8,
    rank: u8,
}

impl traits::ChessSquare for ChessSquare {
    fn file(&self) -> u8 {
        self.file
    }

    fn rank(&self) -> u8 {
        self.rank
    }
}

impl fmt::Display for ChessSquare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            notation::file(self.file).unwrap_or('#'),
            notation::rank(self.rank).unwrap_or('0')
        )
    }
}

impl ChessSquare {
    /// Chess square at `file` and `rank`
    ///
    /// # Panics
    /// Panics if file and/or rank are not between 0-7 inclusive
    pub fn new(file: u8, rank: u8) -> Self {
        assert!(
            (0..8).contains(&file),
            "File must be between 0-7 inclusive, {file} > 7"
        );
        assert!(
            (0..8).contains(&rank),
            "Rank must be between 0-7 inclusive, {file} > 7"
        );
        Self { file, rank }
    }
}

/// Chess move from src to dest
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChessMove {
    src: ChessSquare,
    dest: ChessSquare,
}

impl traits::ChessMove for ChessMove {
    fn src(&self) -> impl traits::ChessSquare {
        self.src
    }

    fn dest(&self) -> impl traits::ChessSquare {
        self.dest
    }
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.src, self.dest)
    }
}

impl ChessMove {
    /// Chess move from `src` to `dest`
    ///
    /// # Panics
    /// Panics if source and destination are the same square
    pub fn new(src: ChessSquare, dest: ChessSquare) -> Self {
        assert_ne!(
            src, dest,
            "Chess move cannot originate and terminate at same square"
        );
        Self { src, dest }
    }
}

/// Chess piece representation
///
/// Internally contains position as well as piece kind and colour
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChessPiece {
    square: ChessSquare,
    kind: PieceKind,
    colour: PieceColour,
}

impl traits::ChessPiece for ChessPiece {
    fn kind(&self) -> PieceKind {
        self.kind
    }

    fn colour(&self) -> PieceColour {
        self.colour
    }
}

impl ChessPiece {
    /// Chess piece
    pub fn new(square: ChessSquare, kind: PieceKind, colour: PieceColour) -> Self {
        Self {
            square,
            kind,
            colour,
        }
    }

    /// The square the piece sits on
    pub fn square(&self) -> ChessSquare {
        self.square
    }

    /// Move piece to `dest`
    ///
    /// Moving a piece to the square it already sits on is defined and will succeed but is usually
    /// indictive of a malfunction in the caller, since this is not a valid chess move.
    pub fn move_piece(&mut self, dest: ChessSquare) {
        if self.square == dest {
            event!(
                Level::WARN,
                "Moved piece {:?} from {} to same square {}",
                self,
                self.square,
                dest
            );
        }
        self.square = dest;
    }
}

/// Piece list representation of chess board
pub struct ChessBoard {
    pieces: Vec<ChessPiece>,
    turn: PieceColour,
    en_passant: Option<ChessSquare>,
    castling_rights: [bool; 4],
}

impl traits::ChessBoard<ChessSquare, ChessPiece, ChessMove> for ChessBoard {
    fn starting_board() -> Self {
        todo!()
    }

    fn get_piece(&self, square: ChessSquare) -> Option<ChessPiece> {
        let mut pieces = self.pieces.iter().filter(|&&piece| piece.square == square);
        let piece = pieces.next().copied();
        assert!(
            pieces.next().is_none(),
            "Two pieces on same square, board state invalid"
        );
        piece
    }

    fn all_pieces(&self) -> impl Iterator<Item = ChessPiece> {
        self.pieces.iter().copied()
    }

    fn move_piece(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        let mut pieces = self
            .pieces
            .iter_mut()
            .filter(|piece| piece.square == chess_move.src);
        let piece = pieces.next();
        assert!(
            pieces.next().is_none(),
            "Two pieces on same square, board state invalid"
        );
        match piece {
            Some(p) => {
                p.move_piece(chess_move.dest);
                Ok(())
            }
            None => Err(ChessError::PieceNotFound(format!("{}", chess_move.src))),
        }
    }
}
