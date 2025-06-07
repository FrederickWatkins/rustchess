//! Piece list representation of a chess board
//!
//! Uses vectors of pieces to represent the chess board. This implementation is far simpler and
//! doesn't involve bit-twiddling, so is less likely to contain bugs, but is very memory heavy and
//! slow.

use core::fmt;
use std::ops::{Add, Div, Sub};

use crate::enums::{PieceColour, PieceKind};
use crate::error::ChessError;
use crate::parser::fen::Fen;
use crate::simple_types::{SimpleMove, SimpleSquare};
use crate::traits::{ChessBoard as _, ChessMove as _, ChessPiece as _, ChessSquare as _};
use crate::{notation, traits};

use itertools::Itertools as _;
use tracing::{Level, event};

/// Offset between two chess squares
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SquareOffset {
    file: i8,
    rank: i8,
}

impl Div<i8> for SquareOffset {
    type Output = SquareOffset;

    fn div(self, rhs: i8) -> Self::Output {
        Self {
            file: self.file / rhs,
            rank: self.rank / rhs,
        }
    }
}

impl SquareOffset {
    fn new(file: i8, rank: i8) -> Self {
        Self { file, rank }
    }
}

impl Add<SquareOffset> for SimpleSquare {
    type Output = SimpleSquare;

    fn add(self, rhs: SquareOffset) -> Self::Output {
        let file = self.file() as i8 + rhs.file;
        let rank = self.rank() as i8 + rhs.rank;
        assert!(file >= 0, "File cannot be offset to less than zero {file} < 0");
        assert!(rank >= 0, "Rank cannot be offset to less than zero {rank} < 0");
        Self::new(file as u8, rank as u8)
    }
}

impl Sub for SimpleSquare {
    type Output = SquareOffset;

    fn sub(self, rhs: Self) -> Self::Output {
        SquareOffset::new(
            self.file() as i8 - rhs.file() as i8,
            self.rank() as i8 - rhs.rank() as i8,
        )
    }
}

/// Chess piece representation
///
/// Internally contains position as well as piece kind and colour
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChessPiece {
    square: SimpleSquare,
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

impl fmt::Display for ChessPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_fen())
    }
}

impl ChessPiece {
    /// Chess piece
    pub fn new(square: SimpleSquare, kind: PieceKind, colour: PieceColour) -> Self {
        Self { square, kind, colour }
    }

    /// The square the piece sits on
    pub fn square(&self) -> SimpleSquare {
        self.square
    }

    /// Move piece to `dest`
    ///
    /// Moving a piece to the square it already sits on is defined and will succeed but is usually
    /// indictive of a malfunction in the caller, since this is not a valid chess move.
    pub fn move_piece(&mut self, dest: SimpleSquare) {
        if self.square == dest {
            event!(Level::WARN, "Moved piece {self:?} from {dest} to same square {dest}");
        }
        self.square = dest;
    }
}

/// Piece list representation of chess board
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChessBoard {
    pieces: Vec<ChessPiece>,
    turn: PieceColour,
    en_passant: Option<SimpleSquare>,
    castling_rights: [bool; 4],
}

impl traits::ChessBoard<SimpleSquare, ChessPiece, SimpleMove> for ChessBoard {
    fn from_fen_internal(fen: Fen) -> Self {
        let mut pieces: Vec<ChessPiece> = vec![];
        for (i, rank) in fen.layout.into_iter().enumerate() {
            for (j, piece) in rank.into_iter().enumerate() {
                if let Some(piece) = piece {
                    pieces.push(ChessPiece::new(
                        SimpleSquare::new(j as u8, 7 - i as u8),
                        piece.kind(),
                        piece.colour(),
                    ));
                }
            }
        }

        Self {
            pieces,
            turn: fen.turn,
            en_passant: fen.en_passant,
            castling_rights: fen.castling_rights,
        }
    }

    fn get_piece(&self, square: SimpleSquare) -> Result<ChessPiece, ChessError> {
        let pieces = self.pieces.iter().filter(|&&piece| piece.square() == square);
        match pieces.at_most_one() {
            Ok(Some(piece)) => Ok(*piece),
            Ok(None) => Err(ChessError::PieceNotFound(square)),
            Err(_) => Err(ChessError::InvalidBoard(format!("Two pieces found at {square}"))),
        }
    }

    fn all_pieces(&self) -> impl Iterator<Item = ChessPiece> {
        self.pieces.iter().copied()
    }

    fn move_piece(&mut self, chess_move: SimpleMove) -> Result<(), ChessError> {
        const PAWN_DOUBLE_PUSH: i8 = 2;
        let taken_piece = self.pieces.iter().position(|piece| piece.square() == chess_move.dest());

        let piece = self.get_piece_mut(chess_move.src())?;
        piece.move_piece(chess_move.dest());
        if let Some(promote_to) = chess_move.promote_to() {
            piece.kind = promote_to;
        }
        let piece = piece.to_owned();
        // Wait till after moving piece succeeds to take
        if let Some(taken_index) = taken_piece {
            self.pieces.remove(taken_index);
        }

        let offset = chess_move.dest() - chess_move.src();
        self.castle_rook(piece, offset)?;

        self.take_en_passant(piece, offset)?;

        if piece.kind() == PieceKind::Pawn && offset.rank.abs() == PAWN_DOUBLE_PUSH {
            self.en_passant = Some(chess_move.src() + offset / 2);
        } else {
            self.en_passant = None;
        }

        self.turn = !self.turn;
        Ok(())
    }
}

impl ChessBoard {
    /// Mutable reference to piece on `square`
    fn get_piece_mut(&mut self, square: SimpleSquare) -> Result<&mut ChessPiece, ChessError> {
        let pieces = self
            .pieces
            .iter_mut()
            .filter(|piece| piece.square() == square)
            .peekable();
        match pieces.at_most_one() {
            Ok(Some(piece)) => Ok(piece),
            Ok(None) => Err(ChessError::PieceNotFound(square)),
            Err(_) => Err(ChessError::InvalidBoard(format!("Two pieces found at {square}"))),
        }
    }

    /// Check if king move was a castle and if so move rook
    fn castle_rook(&mut self, piece: ChessPiece, offset: SquareOffset) -> Result<(), ChessError> {
        const KINGSIDE_CASTLE: i8 = 2;
        const QUEENSIDE_CASTLE: i8 = -3;
        if piece.kind() == PieceKind::King && offset.file == KINGSIDE_CASTLE {
            let rook = self.get_piece_mut(piece.square() + SquareOffset::new(1, 0))?;
            rook.move_piece(piece.square() + SquareOffset::new(-1, 0));
        }
        if piece.kind() == PieceKind::King && offset.file == QUEENSIDE_CASTLE {
            let rook = self.get_piece_mut(piece.square() + SquareOffset::new(-1, 0))?;
            rook.move_piece(piece.square() + SquareOffset::new(1, 0));
        }
        Ok(())
    }

    /// Check if move was en passant and if so take other pawn
    fn take_en_passant(&mut self, piece: ChessPiece, offset: SquareOffset) -> Result<(), ChessError> {
        if let Some(taken_pawn_square) = self.en_passant_target(piece, offset) {
            if let Some(taken_pawn) = self.pieces.iter().position(|piece| piece.square() == taken_pawn_square) {
                self.pieces.remove(taken_pawn);
            } else {
                return Err(ChessError::InvalidBoard(format!(
                    "En passant square present at {} but no pawn to take at {}",
                    piece.square(),
                    taken_pawn_square
                )));
            }
        }
        Ok(())
    }

    /// Check if move was en passant and if so return square of pawn to take
    fn en_passant_target(&self, piece: ChessPiece, offset: SquareOffset) -> Option<SimpleSquare> {
        match self.en_passant {
            Some(en_passant) if piece.kind() == PieceKind::Pawn && piece.square() == en_passant => {
                Some(piece.square() + SquareOffset::new(0, -offset.rank))
            }
            _ => None,
        }
    }

    fn fmt_board(&self) -> String {
        let mut outstr = String::with_capacity(172);
        for i in (0..8).rev() {
            outstr.push(notation::rank_to_char(i).unwrap());
            for j in 0..8 {
                outstr.push(' ');
                if let Ok(piece) = self.get_piece(SimpleSquare::new(j, i)) {
                    outstr.push(piece.as_fen());
                } else if (i + j) % 2 == 1 {
                    outstr.push('☐');
                } else {
                    outstr.push(' ');
                }
            }
            outstr.push('\n');
        }

        outstr.push_str("  ");
        for j in 0..8 {
            outstr.push(notation::file_to_char(j).unwrap());
            outstr.push(' ');
        }
        outstr
    }
}

impl fmt::Display for ChessBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fmt_board())
    }
}

#[cfg(test)]
mod tests {

    use crate::traits::ChessBoard as _;

    use super::*;

    #[test]
    fn normal_square() {
        let square = SimpleSquare::new(5, 4);
        assert_eq!(square.file(), 5);
        assert_eq!(square.rank(), 4);
    }

    #[test]
    #[should_panic(expected = "Rank must be between 0-7 inclusive, 8 > 7")]
    fn wrong_range_square() {
        let _ = SimpleSquare::new(3, 8);
    }

    #[test]
    #[should_panic(expected = "Chess move cannot originate and terminate at same square")]
    fn duplicate_move() {
        let _ = SimpleMove::new(SimpleSquare::new(3, 4), SimpleSquare::new(3, 4), None);
    }

    #[test]
    fn two_on_same_square() {
        let square = SimpleSquare::new(3, 2);
        let board = ChessBoard {
            pieces: vec![
                ChessPiece::new(square, PieceKind::Knight, PieceColour::Black),
                ChessPiece::new(square, PieceKind::Bishop, PieceColour::White),
            ],
            turn: PieceColour::White,
            en_passant: None,
            castling_rights: [false, false, false, false],
        };
        let e = board.get_piece(square).unwrap_err();
        match e {
            ChessError::InvalidBoard(s) => assert_eq!(s, format!("Two pieces found at {square}")),
            _ => panic!("Wrong error type {e}"),
        }
    }

    #[test]
    fn none_on_square() {
        let square = SimpleSquare::new(3, 2);
        let board = ChessBoard {
            pieces: vec![],
            turn: PieceColour::White,
            en_passant: None,
            castling_rights: [false, false, false, false],
        };
        let e = board.get_piece(square).unwrap_err();
        match e {
            ChessError::PieceNotFound(_) => (),
            _ => panic!("Wrong error type {e}"),
        }
    }
}
