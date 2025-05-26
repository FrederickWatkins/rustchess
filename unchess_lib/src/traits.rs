use crate::{
    error::*,
    piece::{Colour, Piece},
    types::*,
};
use rayon::prelude::*;

/// Strict legal move generator
pub trait LegalMoveGenerator: Board + PLegalMoveGenerator + Clone + Sync {
    /// Get all strictly legal moves for piece on board
    fn all_legal_moves(&self) -> Vec<ChessMove> {
        self.all_plegal_moves()
            .into_par_iter() // Could ultimately make slower, need to check
            .filter(|chess_move| {
                let mut temp_board = self.clone();
                temp_board.move_piece(*chess_move).unwrap();
                temp_board.check_king_safe(self.turn())
            })
            .collect()
    }

    /// Get all strictly legal moves for piece on `pos`
    fn piece_legal_moves(&self, pos: Position) -> Result<Vec<ChessMove>, ChessError> {
        Ok(self
            .piece_plegal_moves(pos)?
            .into_iter()
            .filter(|chess_move| {
                let mut temp_board = self.clone();
                temp_board.move_piece(*chess_move).unwrap();
                temp_board.check_king_safe(self.turn())
            })
            .collect())
    }

    /// Check moving a piece from `start` to `end` is strictly legal
    fn check_move_legal(&self, chess_move: ChessMove) -> Result<bool, ChessError> {
        Ok(self.check_move_plegal(chess_move)? && {
            let mut temp_board = self.clone();
            temp_board.move_piece(chess_move).unwrap();
            temp_board.check_king_safe(self.turn())
        })
    }

    /// Move a piece from `start` to `end`, checking for strict legality
    fn move_piece_checked(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        if self.check_move_legal(chess_move)? {
            self.move_piece(chess_move)
        } else {
            Err(ChessError::IllegalMove(chess_move))
        }
    }

    fn check_king_safe(&self, colour: Colour) -> bool;

    fn disambiguate_move(&self, amb_move: AmbiguousMove) -> Result<ChessMove, ChessError> {
        match amb_move {
            AmbiguousMove::Standard {
                end,
                kind,
                start_file,
                start_rank,
                promote,
            } => {
                let moves: Vec<ChessMove> = self
                    .all_legal_moves()
                    .into_iter()
                    .filter(|chess_move| {
                        chess_move.end == end
                            && self.get_piece(chess_move.start).unwrap().kind == kind
                            && match start_file {
                                Some(file) => chess_move.start.0 == file,
                                None => true,
                            }
                            && match start_rank {
                                Some(rank) => chess_move.start.1 == rank,
                                None => true,
                            }
                            && chess_move.promote == promote
                    })
                    .collect();
                if moves.len() > 1 {
                    return Err(ChessError::UnderdefinedMove(amb_move));
                }
                match moves.iter().next() {
                    Some(chess_move) => Ok(*chess_move),
                    None => Err(ChessError::ImpossibleMove(amb_move)),
                }
            }
            AmbiguousMove::Castle(castling_side) => Ok(match castling_side {
                CastlingSide::QueenSide => ChessMove {
                    start: Position(4, self.turn().back_rank()),
                    end: Position(1, self.turn().back_rank()),
                    promote: None,
                },
                CastlingSide::KingSide => ChessMove {
                    start: Position(4, self.turn().back_rank()),
                    end: Position(6, self.turn().back_rank()),
                    promote: None,
                },
            }),
        }
    }

    fn get_board_state(&self) -> BoardState {
        if !self.all_legal_moves().is_empty() {
            if self.check_king_safe(self.turn()) {
                BoardState::Normal
            } else {
                BoardState::Check
            }
        } else {
            if self.check_king_safe(self.turn()) {
                BoardState::Stalemate
            } else {
                BoardState::Checkmate
            }
        }
    }
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
    /// Return board in standard chess starting position
    fn starting_board() -> Self;

    /// Get piece at `pos`
    fn get_piece(&self, pos: Position) -> Option<&Piece>;

    /// Move a piece from `start` to `end` without checking for legality
    fn move_piece(&mut self, chess_move: ChessMove) -> Result<(), ChessError>;

    /// Generate from Forsyth-Edwards Notation
    fn from_fen(fen: &str) -> Result<Self, ChessError>;

    /// Return colour of current turn
    fn turn(&self) -> Colour;
}

/// Chess Game
///
/// The Game trait includes behaviours that rely on the full history of the game, rather than just
/// the current state of the board.
pub trait Game<B>: Board {
    /// Get current state of board in game
    fn current_board(&self) -> &B;

    /// Undo previous move (move one node up tree)
    fn undo_move(&mut self) -> Result<(), ChessError>;

    /// Generate from Portable Game Notation
    fn from_pgn(pgn: &str) -> Result<Self, ChessError>;
}
