use crate::{error::*, types::*};
use rayon::prelude::*;

/// Strict legal move generator
pub trait LegalMoveGenerator: Board + PLegalMoveGenerator + Clone + Sync {
    /// Get all strictly legal moves for piece on board
    fn all_legal_moves(&self) -> Result<Vec<ChessMove>, ChessError> {
        Ok(self
            .all_plegal_moves()
            .par_iter() // Could ultimately make slower, need to check
            .filter(|chess_move| self.check_king_safe(**chess_move).unwrap())
            .map(|chess_move| *chess_move)
            .collect())
    }

    /// Get all strictly legal moves for piece on `pos`
    fn piece_legal_moves(&self, pos: Position) -> Result<Vec<ChessMove>, ChessError> {
        Ok(self
            .piece_plegal_moves(pos)?
            .iter()
            .filter(|chess_move| self.check_king_safe(**chess_move).unwrap())
            .map(|chess_move| *chess_move)
            .collect())
    }

    /// Check moving a piece from `start` to `end` is strictly legal
    fn check_move_legal(&self, chess_move: ChessMove) -> Result<bool, ChessError> {
        Ok(self.check_move_plegal(chess_move)? && self.check_king_safe(chess_move)?)
    }

    /// Move a piece from `start` to `end`, checking for strict legality
    fn move_piece_checked(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        if self.check_move_legal(chess_move)? {
            self.move_piece(chess_move)
        } else {
            Err(ChessError::IllegalMove(chess_move))
        }
    }

    fn check_king_safe(&self, chess_move: ChessMove) -> Result<bool, ChessError>;
}

/// Pseudo legal move generator
pub trait PLegalMoveGenerator: Board {
    /// Get all pseudo-legal moves for piece on board
    fn all_plegal_moves(&self) -> Vec<ChessMove>;

    /// Get all pseudo-legal moves for piece on `pos`
    fn piece_plegal_moves(&self, pos: Position) -> Result<Vec<ChessMove>, ChessError>;

    /// Check moving a piece from `start` to `end` is pseudo-legal
    fn check_move_plegal(&self, chess_move: ChessMove) -> Result<bool, ChessError>;

    /// Move a piece from `start` to `end`, checking for pseudo-legality
    fn move_piece_pchecked(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        if self.check_move_plegal(chess_move)? {
            self.move_piece(chess_move)
        } else {
            Err(ChessError::IllegalMove(chess_move))
        }
    }
}

/// Chess Board
pub trait Board: Sized {
    /// Move a piece from `start` to `end` without checking for legality
    fn move_piece(&mut self, chess_move: ChessMove) -> Result<(), ChessError>;

    /// Generate from Forsyth-Edwards Notation
    fn from_fen(fen: &str) -> Result<Self, ChessError>;
}

/// Chess Game
///
/// The Game trait includes behaviours that rely on the full history of the game, rather than just
/// the current state of the board.
pub trait Game: Board {
    /// Undo previous move (move one node up tree)
    fn undo_move(&mut self) -> Result<(), ChessError>;

    /// Generate from Portable Game Notation
    fn from_pgn(pgn: &str) -> Result<Self, ChessError>;
}
