//! Shared traits used across crate
//!
//! Since the different methods of representing a chess board have a wide variety of different
//! internal representations, many traits are provided for easing compatibility while maintaining
//! strong performance. For example, the [`ChessSquare`] trait is necessary since if the public API
//! assumed that its internal representation was two integers, this would create massive overhead
//! when interacting with bitboards.
use std::fmt::{Debug, Display};

use crate::{
    enums::{BoardState, PieceColour, PieceKind},
    error::ChessError,
};

/// Generic chess square
///
/// Can't be a transparent shared data type because of differences in internal board
/// representations, so setters and getters must be used instead.
pub trait ChessSquare: Debug + Display + Copy {
    /// File of the square
    ///
    /// Returns a value from 0-7 inclusive where 0 represents the a-file and 7 the h-file.
    fn file(&self) -> u8;

    /// Rank of the square
    ///
    /// Returns a value from 0-7 inclusive where 0 represents the 1st rank and 7 the 8th rank.
    fn rank(&self) -> u8;
}

/// Generic unambiguous chess move
///
/// Can't be a transparent shared data type because of differences in internal board
/// representations, so setters and getters must be used instead.
///
/// For an ambiguous chess move datatype compatible with PGN notation, see TODO
pub trait ChessMove: Debug + Display + Copy {
    /// Source square of the chess move
    fn src(&self) -> impl ChessSquare;

    /// Destination square of the chess move
    fn dest(&self) -> impl ChessSquare;
}

/// Generic piece
///
/// Does not include positions such as would be required for mailboxing since it would be
/// unnecessary for bitboards.
pub trait ChessPiece {
    /// The type of the piece
    fn kind(&self) -> PieceKind;

    /// The colour of the piece
    fn colour(&self) -> PieceColour;
}

/// Chess board
///
/// Implies no ability to check the legality of moves or to ensure that the board is in a valid
/// state, just the ability to store and manipulate internal state
pub trait ChessBoard {
    /// Return the default starting chess board
    ///
    /// White's turn, all pieces in starting positions, with castling rights.
    fn starting_board() -> Self;

    /// Return piece at `square`
    ///
    /// Returns none if no piece present.
    fn get_piece(&self, square: impl ChessSquare) -> Option<impl ChessPiece>;

    /// Return iterator over all pieces on board
    ///
    /// No guaranteed order.
    fn all_pieces(&self) -> impl Iterator<Item = impl ChessMove>;

    /// Moves a piece on the chess board
    ///
    /// Move piece without checking for any any kind of legality, but updating state (turn, en
    /// passant ability, etc).
    ///
    /// # Errors
    /// - [`crate::error::ChessError::PieceNotFound`] if no piece present at `chess_move.src()`
    fn move_piece(&mut self, chess_move: impl ChessMove) -> Result<(), ChessError>;
}

/// Pseudo-legal move generator
///
/// Capable of generating pseudo-legal moves e.g. moves that satisfy the fundemental requirements of
/// where a piece may move, but do not necessarily leave the board in a valid state after they are
/// executed e.g. the king may be left in check. Associated functions generally much faster than
/// [`LegalMoveGenerator`].
pub trait PLegalMoveGenerator: ChessBoard {
    /// Return all pseudo-legal moves from the current board state
    ///
    /// Will not check for leaving the king in check, if strict legality is necessary then use
    /// [`LegalMoveGenerator::all_legal_moves`].
    ///
    /// # Errors
    /// - [`crate::error::ChessError::InvalidBoard`] if the board is in an invalid state, for example if
    ///   there are no pieces of the colour of the current turn or there is not one king of each
    ///   colour on the board.
    fn all_plegal_moves(&self) -> Result<impl Iterator<Item = impl ChessMove>, ChessError>;

    /// Return all pseudo-legal moves for the piece at `square`
    ///
    /// Will not check for leaving the king in check, if strict legality is necessary then use
    /// [`LegalMoveGenerator::piece_legal_moves`].
    ///
    /// # Errors
    /// - [`crate::error::ChessError::InvalidBoard`] if the board is in an invalid state, for example if
    ///   there are no pieces of the colour of the current turn or there is not one king of each
    ///   colour on the board.
    /// - [`crate::error::ChessError::PieceNotFound`] if no piece present at `square`
    fn piece_plegal_moves(
        &self,
        square: impl ChessSquare,
    ) -> Result<impl Iterator<Item = impl ChessMove>, ChessError>;

    /// Return true if move `chess_move` is pseudo-legal
    ///
    /// Will not check for leaving the king in check, if strict legality is necessary then use
    /// [`LegalMoveGenerator::is_move_legal`].
    ///
    /// # Errors
    /// - [`crate::error::ChessError::InvalidBoard`] if the board is in an invalid state, for example if
    ///   there are no pieces of the colour of the current turn or there is not one king of each
    ///   colour on the board.
    /// - [`crate::error::ChessError::PieceNotFound`] if no piece present at `chess_move.src()`
    fn is_move_plegal(&self, chess_move: impl ChessMove) -> Result<bool, ChessError>;

    /// Move piece if move is pseudo-legal, otherwise error
    ///
    /// Will not check for leaving the king in check, if strict legality is necessary then use
    /// [`LegalMoveGenerator::is_move_legal`].
    ///
    /// # Errors
    /// - [`crate::error::ChessError::InvalidBoard`] if the board is in an invalid state, for example if
    ///   there are no pieces of the colour of the current turn or there is not one king of each
    ///   colour on the board.
    /// - [`crate::error::ChessError::PieceNotFound`] if no piece present at `chess_move.src()`
    /// - [`crate::error::ChessError::IllegalMove`] if chess_move is illegal
    fn move_piece_plegal(&mut self, chess_move: impl ChessMove) -> Result<(), ChessError> {
        if self.is_move_plegal(chess_move)? {
            self.move_piece(chess_move)
        } else {
            Err(ChessError::IllegalMove(format!("{chess_move}")))
        }
    }
}

/// Strict legal move generator
///
/// Capable of generating strictly legal moves e.g. moves that both fulfil pieces' individual
/// movement requirements and do not leave the king in check.
pub trait LegalMoveGenerator: PLegalMoveGenerator {
    /// Return all legal moves from the current board state
    ///
    /// # Errors
    /// - [`crate::error::ChessError::InvalidBoard`] if the board is in an invalid state, for example if
    ///   there are no pieces of the colour of the current turn or there is not one king of each
    ///   colour on the board.
    fn all_legal_moves(&self) -> Result<impl Iterator<Item = impl ChessMove>, ChessError>;

    /// Return all legal moves for the piece at `square`
    ///
    /// # Errors
    /// - [`crate::error::ChessError::InvalidBoard`] if the board is in an invalid state, for example if
    ///   there are no pieces of the colour of the current turn or there is not one king of each
    ///   colour on the board.
    /// - [`crate::error::ChessError::PieceNotFound`] if no piece present at `square`
    fn piece_legal_moves(
        &self,
        square: impl ChessSquare,
    ) -> Result<impl Iterator<Item = impl ChessMove>, ChessError>;

    /// Return true if move `chess_move` is legal
    ///
    /// # Errors
    /// - [`crate::error::ChessError::InvalidBoard`] if the board is in an invalid state, for example if
    ///   there are no pieces of the colour of the current turn or there is not one king of each
    ///   colour on the board.
    /// - [`crate::error::ChessError::PieceNotFound`] if no piece present at `chess_move.src()`
    fn is_move_legal(&self, chess_move: impl ChessMove) -> Result<bool, ChessError>;

    /// Move piece if move is legal, otherwise error
    ///
    /// # Errors
    /// - [`crate::error::ChessError::InvalidBoard`] if the board is in an invalid state, for example if
    ///   there are no pieces of the colour of the current turn or there is not one king of each
    ///   colour on the board.
    /// - [`crate::error::ChessError::PieceNotFound`] if no piece present at `chess_move.src()`
    /// - [`crate::error::ChessError::IllegalMove`] if chess_move is illegal
    fn move_piece_legal(&mut self, chess_move: impl ChessMove) -> Result<(), ChessError>{
        if self.is_move_legal(chess_move)? {
            self.move_piece(chess_move)
        } else {
            Err(ChessError::IllegalMove(format!("{chess_move}")))
        }
    }

    /// Get current board state
    ///
    /// # Errors
    /// - [`crate::error::ChessError::InvalidBoard`] if the board is in an invalid state, for example if
    ///   there are no pieces of the colour of the current turn or there is not one king of each
    ///   colour on the board.
    fn state(&self) -> Result<BoardState, ChessError>;
}
