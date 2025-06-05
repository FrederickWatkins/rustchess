//! Module for enums common to all chess board representations

use core::fmt;
use std::ops::Not;

use crate::{default_types::SimpleSquare, error::ChessError, notation, pgn, traits::ChessSquare as _};

/// Colour of piece
#[allow(missing_docs)] // Enum variants self explanatory
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PieceColour {
    Black,
    White,
}

impl Not for PieceColour {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            PieceColour::Black => PieceColour::White,
            PieceColour::White => PieceColour::Black,
        }
    }
}

/// Type of piece
#[allow(missing_docs)] // Enum variants self explanatory
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PieceKind {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

impl From<PieceKind> for char {
    fn from(value: PieceKind) -> Self {
        match value {
            PieceKind::King => 'K',
            PieceKind::Queen => 'Q',
            PieceKind::Bishop => 'B',
            PieceKind::Knight => 'N',
            PieceKind::Rook => 'R',
            PieceKind::Pawn => 'P',
        }
    }
}

/// Basic states of board based on king safety
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BoardState {
    /// Normal play in game, no restrictions on moves
    Normal,
    /// King is in check, only legal moves are ones that break the check
    Check,
    /// Game is over in a stalemate, king has no legal moves but not in check
    Stalemate,
    /// Game is over in a checkmate, king has no legal moves and is checked
    Checkmate,
}

impl From<MoveAction> for BoardState {
    fn from(value: MoveAction) -> Self {
        match value {
            MoveAction::Check => Self::Check,
            MoveAction::Checkmate => Self::Checkmate,
        }
    }
}

/// Action caused by move
#[allow(missing_docs)] // Enum variants self explanatory
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MoveAction {
    Check,
    Checkmate,
}

impl From<MoveAction> for char {
    fn from(value: MoveAction) -> Self {
        match value {
            MoveAction::Check => '+',
            MoveAction::Checkmate => '#',
        }
    }
}

impl TryFrom<BoardState> for MoveAction {
    type Error = ChessError;

    fn try_from(value: BoardState) -> Result<Self, Self::Error> {
        match value {
            BoardState::Check => Ok(Self::Check),
            BoardState::Checkmate => Ok(Self::Checkmate),
            _ => Err(ChessError::NotAction(value)),
        }
    }
}

/// Side to castle on
#[allow(missing_docs)] // Enum variants self explanatory
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CastlingSide {
    KingSide,
    QueenSide,
}

impl CastlingSide {
    /// Return as string in PGN format
    pub fn as_str(&self) -> &str {
        match self {
            CastlingSide::QueenSide => "O-O-O",
            CastlingSide::KingSide => "O-O",
        }
    }
}

/// Ambiguous move, pgn standard
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AmbiguousMove {
    /// Normal move
    Normal {
        /// The type of piece moving
        piece_kind: PieceKind,
        /// The file disambiguation for source if present
        src_file: Option<u8>,
        /// The rank disambiguation for source if present
        src_rank: Option<u8>,
        /// Whether the move takes a piece or not
        takes: bool,
        /// The destination square
        dest: SimpleSquare,
        /// The piece to be promoted to
        promote_to: Option<PieceKind>,
        /// The action resulting from the move (check, checkmate)
        action: Option<MoveAction>,
    },
    /// Castling
    Castle {
        /// Side to castle on
        side: CastlingSide,
    },
}

impl AmbiguousMove {
    /// Convert to string according to pgn standard
    ///
    /// # Panics
    /// Panics if src_file or src_range are out of range
    pub fn as_pgn_str(&self) -> String {
        match self {
            AmbiguousMove::Normal {
                piece_kind,
                src_file,
                src_rank,
                takes,
                dest,
                promote_to,
                action,
            } => {
                let mut s = String::new();
                if *piece_kind != PieceKind::Pawn {
                    s.push(char::from(*piece_kind));
                }
                if let Some(f) = src_file {
                    s.push(notation::file(*f).unwrap());
                }
                if let Some(r) = src_rank {
                    s.push(notation::rank(*r).unwrap());
                }
                if *takes {
                    s.push('x');
                }
                s.push_str(&dest.as_str());
                if let Some(p) = promote_to {
                    s.push('=');
                    s.push(char::from(*p));
                }
                if let Some(a) = action {
                    s.push(char::from(*a));
                }
                s
            }
            AmbiguousMove::Castle { side } => side.as_str().to_string(),
        }
    }
}

impl fmt::Display for AmbiguousMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_pgn_str())
    }
}

impl TryFrom<&str> for AmbiguousMove {
    type Error = ChessError;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok((_, chess_move)) = pgn::chess_move(value) {
            Ok(chess_move)
        } else {
            Err(ChessError::InvalidPGN(value.to_string()))
        }
    }
}
