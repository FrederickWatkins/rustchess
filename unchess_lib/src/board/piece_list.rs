//! Piece list representation of a chess board
//!
//! Uses vectors of pieces to represent the chess board. This implementation is far simpler and
//! doesn't involve bit-twiddling, so is less likely to contain bugs, but is very memory heavy and
//! slow.

use std::fmt;
use std::ops::{Add, Div, Sub};

use crate::enums::{PieceColour, PieceKind};
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

impl Add<SquareOffset> for ChessSquare {
    type Output = ChessSquare;

    fn add(self, rhs: SquareOffset) -> Self::Output {
        let file = self.file as i8 + rhs.file;
        let rank = self.rank as i8 + rhs.rank;
        assert!(file >= 0, "File cannot be offset to less than zero {file} < 0");
        assert!(rank >= 0, "Rank cannot be offset to less than zero {rank} < 0");
        Self::new(file as u8, rank as u8)
    }
}

impl Sub for ChessSquare {
    type Output = SquareOffset;

    fn sub(self, rhs: Self) -> Self::Output {
        SquareOffset::new(self.file as i8 - rhs.file as i8, self.rank as i8 - rhs.rank as i8)
    }
}

impl ChessSquare {
    /// Chess square at `file` and `rank`
    ///
    /// # Panics
    /// Panics if file and/or rank are not between 0-7 inclusive
    pub fn new(file: u8, rank: u8) -> Self {
        assert!((0..8).contains(&file), "File must be between 0-7 inclusive, {file} > 7");
        assert!((0..8).contains(&rank), "Rank must be between 0-7 inclusive, {file} > 7");
        Self { file, rank }
    }
}

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

/// Chess move from src to dest
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChessMove {
    src: ChessSquare,
    dest: ChessSquare,
    promote_to: Option<PieceKind>,
}

impl traits::ChessMove<ChessSquare> for ChessMove {
    fn src(&self) -> ChessSquare {
        self.src
    }

    fn dest(&self) -> ChessSquare {
        self.dest
    }

    fn promote_to(&self) -> Option<PieceKind> {
        self.promote_to
    }
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut outstr = format!("{}{}", self.src, self.dest);
        if let Some(promotion) = self.promote_to {
            outstr.push('=');
            outstr.push(promotion.into());
        }
        write!(f, "{outstr}")
    }
}

impl ChessMove {
    /// Chess move from `src` to `dest`
    ///
    /// # Panics
    /// Panics if source and destination are the same square
    pub fn new(src: ChessSquare, dest: ChessSquare, promote_to: Option<PieceKind>) -> Self {
        assert_ne!(src, dest, "Chess move cannot originate and terminate at same square");
        Self { src, dest, promote_to }
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

impl fmt::Display for ChessPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", notation::piece(self.colour, self.kind))
    }
}

impl ChessPiece {
    /// Chess piece
    pub fn new(square: ChessSquare, kind: PieceKind, colour: PieceColour) -> Self {
        Self { square, kind, colour }
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

    fn get_piece(&self, square: ChessSquare) -> Result<ChessPiece, ChessError> {
        let mut pieces = self.pieces.iter().filter(|&&piece| piece.square == square);
        let piece = pieces.next().copied();
        assert!(
            pieces.next().is_none(),
            "Two pieces on same square, board state invalid"
        );
        if let Some(p) = piece {
            Ok(p)
        } else {
            Err(ChessError::PieceNotFound(Box::new(square)))
        }
    }

    fn all_pieces(&self) -> impl Iterator<Item = ChessPiece> {
        self.pieces.iter().copied()
    }

    fn move_piece(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        const PAWN_DOUBLE_PUSH: i8 = 2;
        let taken_piece = self.pieces.iter().position(|piece| piece.square == chess_move.dest);

        let piece = self.get_piece_mut(chess_move.src)?;
        piece.move_piece(chess_move.dest);
        if let Some(promote_to) = chess_move.promote_to {
            piece.kind = promote_to;
        }
        let piece = piece.to_owned();
        // Wait till after moving piece succeeds to take
        if let Some(taken_index) = taken_piece {
            self.pieces.remove(taken_index);
        }

        let offset = chess_move.dest - chess_move.src;
        self.castle_rook(piece, offset)?;

        match self.en_passant {
            Some(en_passant) if piece.kind == PieceKind::Pawn && chess_move.dest == en_passant => {
                self.take_en_passant(piece, offset)?;
            }
            _ => (),
        }
        if piece.kind == PieceKind::Pawn && offset.rank.abs() == PAWN_DOUBLE_PUSH {
            self.en_passant = Some(chess_move.src + offset / 2);
        } else {
            self.en_passant = None;
        }

        self.turn = !self.turn;
        Ok(())
    }
}

impl ChessBoard {
    fn get_piece_mut(&mut self, square: ChessSquare) -> Result<&mut ChessPiece, ChessError> {
        let mut pieces = self.pieces.iter_mut().filter(|piece| piece.square == square);
        let piece = pieces.next();
        assert!(
            pieces.next().is_none(),
            "Two pieces on same square, board state invalid"
        );
        if let Some(p) = piece {
            Ok(p)
        } else {
            Err(ChessError::PieceNotFound(Box::new(square)))
        }
    }

    /// Check if king move was a castle and if so move rook
    fn castle_rook(&mut self, piece: ChessPiece, offset: SquareOffset) -> Result<(), ChessError> {
        const KINGSIDE_CASTLE: i8 = 2;
        const QUEENSIDE_CASTLE: i8 = -3;
        if piece.kind == PieceKind::King && offset.file == KINGSIDE_CASTLE {
            let rook = self.get_piece_mut(piece.square + SquareOffset::new(1, 0))?;
            rook.move_piece(piece.square + SquareOffset::new(-1, 0));
        }
        if piece.kind == PieceKind::King && offset.file == QUEENSIDE_CASTLE {
            let rook = self.get_piece_mut(piece.square + SquareOffset::new(-1, 0))?;
            rook.move_piece(piece.square + SquareOffset::new(1, 0));
        }
        Ok(())
    }

    fn take_en_passant(&mut self, piece: ChessPiece, offset: SquareOffset) -> Result<(), ChessError> {
        let taken_pawn_square = piece.square + SquareOffset::new(0, -offset.rank);
        if let Some(taken_pawn) = self.pieces.iter().position(|piece| piece.square == taken_pawn_square) {
            self.pieces.remove(taken_pawn);
        } else {
            return Err(ChessError::InvalidBoard(format!(
                "En passant square present at {} but no pawn to take at {}",
                piece.square, taken_pawn_square
            )));
        }
        Ok(())
    }
}
