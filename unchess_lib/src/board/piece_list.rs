//! Piece list representation of a chess board
//!
//! Uses vectors of pieces to represent the chess board. This implementation is far simpler and
//! doesn't involve bit-twiddling, so is less likely to contain bugs, but is very memory heavy and
//! slow.

use core::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Sub};

use crate::enums::{AmbiguousMove, BoardState, CastlingSide, PieceColour, PieceKind};
use crate::error::ChessError;
use crate::parser::fen::Fen;
use crate::simple_types::{SimpleMove, SimpleSquare};
use crate::traits::{
    ChessBoard as _, ChessMove as _, ChessPiece as _, ChessSquare as _, LegalMoveGenerator, PLegalMoveGenerator,
};
use crate::{notation, traits};

use itertools::Itertools as _;

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

impl Mul<PieceColour> for SquareOffset {
    type Output = SquareOffset;

    fn mul(self, rhs: PieceColour) -> Self::Output {
        match rhs {
            PieceColour::White => self,
            PieceColour::Black => Self {
                file: self.file,
                rank: -self.rank,
            },
        }
    }
}

impl SquareOffset {
    const fn new(file: i8, rank: i8) -> Self {
        Self { file, rank }
    }

    fn would_overflow(&self, square: SimpleSquare) -> bool {
        -self.file > square.file() as i8
            || self.file > 7 - square.file() as i8
            || -self.rank > square.rank() as i8
            || self.rank > 7 - square.rank() as i8
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

impl AddAssign<SquareOffset> for SimpleSquare {
    fn add_assign(&mut self, rhs: SquareOffset) {
        *self = self.add(rhs);
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

    /// Is piece at starting rank (for pawns)
    pub fn is_starting_rank(&self) -> bool {
        match self.colour {
            PieceColour::Black if self.square.rank() == 6 => true,
            PieceColour::White if self.square.rank() == 1 => true,
            _ => false,
        }
    }

    fn promotions_on_square(&self, square: SimpleSquare) -> Vec<SimpleMove> {
        if square.rank() == 0 || square.rank() == 7 {
            vec![
                SimpleMove::new(self.square, square, Some(PieceKind::Queen)),
                SimpleMove::new(self.square, square, Some(PieceKind::Knight)),
                SimpleMove::new(self.square, square, Some(PieceKind::Bishop)),
                SimpleMove::new(self.square, square, Some(PieceKind::Rook)),
            ]
        } else {
            vec![SimpleMove::new(self.square, square, None)]
        }
    }

    /// Move piece to `dest`
    ///
    /// Moving a piece to the square it already sits on is defined and will succeed but is usually
    /// indictive of a malfunction in the caller, since this is not a valid chess move.
    pub fn move_piece(&mut self, dest: SimpleSquare) {
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

    fn all_pieces(&self) -> impl IntoIterator<Item = ChessPiece> {
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

const QUEEN_DIRECTIONS: [SquareOffset; 8] = [
    SquareOffset::new(-1, -1), // SW
    SquareOffset::new(-1, 1),  // NW
    SquareOffset::new(1, -1),  // SE
    SquareOffset::new(1, 1),   // NE
    SquareOffset::new(0, -1),  // S
    SquareOffset::new(0, 1),   // N
    SquareOffset::new(-1, 0),  // W
    SquareOffset::new(1, 0),   // E
];

const KNIGHT_PATTERN: [SquareOffset; 8] = [
    SquareOffset::new(-2, -1),
    SquareOffset::new(-2, 1),
    SquareOffset::new(-1, -2),
    SquareOffset::new(-1, 2),
    SquareOffset::new(1, -2),
    SquareOffset::new(1, 2),
    SquareOffset::new(2, -1),
    SquareOffset::new(2, 1),
];

const KING_PATTERN: [SquareOffset; 8] = QUEEN_DIRECTIONS;

impl PLegalMoveGenerator<SimpleSquare, ChessPiece, SimpleMove> for ChessBoard {
    fn all_plegal_moves(&self) -> Result<impl IntoIterator<Item = SimpleMove>, ChessError> {
        let mut out: Vec<SimpleMove> = vec![];
        for piece in self.pieces.iter().filter(|piece| piece.colour == self.turn) {
            match self.piece_plegal_moves(piece.square) {
                Ok(moves) => out.extend(moves),
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }

    fn piece_plegal_moves(&self, square: SimpleSquare) -> Result<impl IntoIterator<Item = SimpleMove>, ChessError> {
        let piece = self.get_piece(square)?;
        if piece.colour != self.turn {
            return Ok(vec![]);
        }
        match piece.kind() {
            PieceKind::King => self.offset_moves(&piece, &KING_PATTERN), // TODO implement castling
            PieceKind::Queen => self.traversal_moves(&piece, &QUEEN_DIRECTIONS),
            PieceKind::Bishop => self.traversal_moves(&piece, &QUEEN_DIRECTIONS[0..4]),
            PieceKind::Knight => self.offset_moves(&piece, &KNIGHT_PATTERN),
            PieceKind::Rook => self.traversal_moves(&piece, &QUEEN_DIRECTIONS[4..8]),
            PieceKind::Pawn => self.pawn_moves(&piece),
        }
    }

    fn is_move_plegal(&self, chess_move: SimpleMove) -> Result<bool, ChessError> {
        Ok(self
            .piece_plegal_moves(chess_move.src())?
            .into_iter()
            .contains(&chess_move))
    }

    fn move_piece_plegal(&mut self, chess_move: SimpleMove) -> Result<(), ChessError> {
        if self.is_move_plegal(chess_move)? {
            self.move_piece(chess_move)?;
            Ok(())
        } else {
            Err(ChessError::IllegalMove(chess_move))
        }
    }
}

impl LegalMoveGenerator<SimpleSquare, ChessPiece, SimpleMove> for ChessBoard {
    fn all_legal_moves(&self) -> Result<impl IntoIterator<Item = SimpleMove>, ChessError> {
        let mut moves: Vec<SimpleMove> = vec![];
        for chess_move in self.all_plegal_moves()? {
            let mut board = self.clone();
            board.move_piece(chess_move)?;
            if !board.king_in_check(self.turn)? {
                moves.push(chess_move);
            }
        }
        Ok(moves)
    }

    fn piece_legal_moves(&self, square: SimpleSquare) -> Result<impl IntoIterator<Item = SimpleMove>, ChessError> {
        let mut moves: Vec<SimpleMove> = vec![];
        for chess_move in self.piece_plegal_moves(square)? {
            let mut board = self.clone();
            board.move_piece(chess_move)?;
            if !board.king_in_check(self.turn)? {
                moves.push(chess_move);
            }
        }
        Ok(moves)
    }

    fn is_move_legal(&self, chess_move: SimpleMove) -> Result<bool, ChessError> {
        if self.is_move_plegal(chess_move)? {
            let mut board = self.clone();
            board.move_piece(chess_move)?;
            if !board.king_in_check(self.turn)? {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn move_piece_legal(&mut self, chess_move: SimpleMove) -> Result<(), ChessError> {
        if self.is_move_legal(chess_move)? {
            self.move_piece(chess_move)?;
            Ok(())
        } else {
            Err(ChessError::IllegalMove(chess_move))
        }
    }

    fn state(&self) -> Result<BoardState, ChessError> {
        match (
            self.all_legal_moves()?.into_iter().try_len().unwrap(),
            self.king_in_check(self.turn)?,
        ) {
            (0, true) => Ok(BoardState::Checkmate),
            (0, false) => Ok(BoardState::Stalemate),
            (_, true) => Ok(BoardState::Check),
            _ => Ok(BoardState::Normal),
        }
    }

    fn disambiguate_move_internal(&self, chess_move: AmbiguousMove) -> Result<SimpleMove, ChessError> {
        match chess_move {
            AmbiguousMove::Normal { .. } => self.disambiguate_normal(chess_move),
            AmbiguousMove::Castle { .. } => Ok(self.disambiguate_castling(chess_move)),
        }
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
        const QUEENSIDE_CASTLE: i8 = -2;
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

    fn pawn_moves(&self, piece: &ChessPiece) -> Result<Vec<SimpleMove>, ChessError> {
        let mut moves: Vec<SimpleMove> = vec![];
        let single_push = piece.square + SquareOffset::new(0, 1) * piece.colour;
        let mut takes = vec![];
        if piece.square.file() > 0 {
            takes.push(piece.square + SquareOffset::new(-1, 1) * piece.colour);
        }
        if piece.square.file() < 7 {
            takes.push(piece.square + SquareOffset::new(1, 1) * piece.colour);
        }
        if self.square_empty(single_push)? {
            moves.append(&mut piece.promotions_on_square(single_push));
            if piece.is_starting_rank() && self.square_empty(piece.square + SquareOffset::new(0, 2) * piece.colour)? {
                moves.push(SimpleMove::new(
                    piece.square,
                    piece.square + SquareOffset::new(0, 2) * piece.colour,
                    None,
                ));
            }
        }
        for take in takes {
            match (self.en_passant, self.get_piece(take)) {
                (_, Ok(other_piece)) if other_piece.colour != piece.colour => {
                    moves.append(&mut piece.promotions_on_square(take));
                }
                (Some(en_passant), Err(ChessError::PieceNotFound(_))) if en_passant == take => {
                    moves.push(SimpleMove::new(piece.square, take, None));
                }
                (_, Err(ChessError::PieceNotFound(_)) | Ok(_)) => (),
                (_, Err(e)) => return Err(e),
            }
        }
        Ok(moves)
    }

    fn traversal_moves(&self, piece: &ChessPiece, directions: &[SquareOffset]) -> Result<Vec<SimpleMove>, ChessError> {
        let mut moves: Vec<SimpleMove> = vec![];
        for direction in directions {
            let mut curr_square = piece.square;
            while !direction.would_overflow(curr_square) {
                curr_square += *direction;
                if self.square_takeable(piece, curr_square)? {
                    moves.push(SimpleMove::new(piece.square, curr_square, None));
                }
                if !self.square_empty(curr_square)? {
                    break;
                }
            }
        }
        Ok(moves)
    }

    fn offset_moves(&self, piece: &ChessPiece, pattern: &[SquareOffset]) -> Result<Vec<SimpleMove>, ChessError> {
        let mut moves: Vec<SimpleMove> = vec![];
        for offset in pattern {
            if offset.would_overflow(piece.square) {
                continue;
            }
            let target_square = piece.square + *offset;
            if self.square_takeable(piece, target_square)? {
                moves.push(SimpleMove::new(piece.square, target_square, None));
            }
        }
        Ok(moves)
    }

    fn square_empty(&self, square: SimpleSquare) -> Result<bool, ChessError> {
        match self.get_piece(square) {
            Ok(_) => Ok(false),
            Err(ChessError::PieceNotFound(_)) => Ok(true),
            Err(e) => Err(e),
        }
    }

    fn square_takeable(&self, piece: &ChessPiece, square: SimpleSquare) -> Result<bool, ChessError> {
        match self.get_piece(square) {
            Ok(other_piece) if other_piece.colour != piece.colour => Ok(true),
            Err(ChessError::PieceNotFound(_)) => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e),
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
                    outstr.push('◼');
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

    fn king_in_check(&self, colour: PieceColour) -> Result<bool, ChessError> {
        if let Ok(king) = self
            .pieces
            .iter()
            .filter(|piece| piece.kind == PieceKind::King && piece.colour == colour)
            .exactly_one()
        {
            self.piece_under_attack(king)
        } else {
            Err(ChessError::InvalidBoard(format!(
                "Number of kings of colour {colour:?} on the board not equal to one"
            )))
        }
    }

    /// Checks if piece is under attack by pretending its other pieces and seeing if it can attack
    ///
    /// Symmetry is beautiful!
    fn piece_under_attack(&self, piece: &ChessPiece) -> Result<bool, ChessError> {
        use traits::ChessMove;
        let mut attacked = self.piece_attacked_by(
            piece,
            self.traversal_moves(piece, &QUEEN_DIRECTIONS[0..4])?
                .iter()
                .map(ChessMove::dest),
            &[PieceKind::Queen, PieceKind::Bishop],
        )?;
        attacked |= self.piece_attacked_by(
            piece,
            self.traversal_moves(piece, &QUEEN_DIRECTIONS[4..8])?
                .iter()
                .map(ChessMove::dest),
            &[PieceKind::Queen, PieceKind::Rook],
        )?;
        attacked |= self.piece_attacked_by(
            piece,
            self.offset_moves(piece, &KNIGHT_PATTERN)?.iter().map(ChessMove::dest),
            &[PieceKind::Knight],
        )?;
        attacked |= self.piece_attacked_by(
            piece,
            self.offset_moves(piece, &KING_PATTERN)?.iter().map(ChessMove::dest),
            &[PieceKind::King],
        )?;
        attacked |= self.piece_attacked_by(
            piece,
            self.pawn_moves(piece)?.iter().map(ChessMove::dest),
            &[PieceKind::Pawn],
        )?;

        Ok(attacked)
    }

    fn piece_attacked_by(
        &self,
        piece: &ChessPiece,
        src_squares: impl Iterator<Item = SimpleSquare>,
        piece_kinds: &[PieceKind],
    ) -> Result<bool, ChessError> {
        for square in src_squares {
            match self.get_piece(square) {
                Ok(ChessPiece { kind, colour, .. }) if colour != piece.colour && piece_kinds.contains(&kind) => {
                    return Ok(true);
                }
                Err(ChessError::PieceNotFound(_)) | Ok(_) => (),
                Err(e) => return Err(e),
            }
        }
        Ok(false)
    }

    fn disambiguate_normal(&self, chess_move: AmbiguousMove) -> Result<SimpleMove, ChessError> {
        let (piece_kind, src_file, src_rank, takes, dest, promote_to, action) = match chess_move {
            AmbiguousMove::Normal {
                piece_kind,
                src_file,
                src_rank,
                takes,
                dest,
                promote_to,
                action,
            } => (piece_kind, src_file, src_rank, takes, dest, promote_to, action),
            AmbiguousMove::Castle { .. } => panic!("Can't use normal move disambiguator on castle"),
        };
        let all_moves: Vec<SimpleMove> = self
            .all_legal_moves()?
            .into_iter()
            .filter(|unambiguous_move| {
                let mut is_match = true;
                is_match &= self.get_piece(unambiguous_move.src()).unwrap().kind() == piece_kind;
                if let Some(file) = src_file {
                    is_match &= unambiguous_move.src().file() == file;
                }
                if let Some(rank) = src_rank {
                    is_match &= unambiguous_move.src().rank() == rank;
                }
                if takes {
                    is_match &= self.get_piece(unambiguous_move.dest()).is_ok();
                }
                is_match &= unambiguous_move.dest() == dest;
                is_match &= unambiguous_move.promote_to() == promote_to;
                if let Some(action) = action {
                    let mut board = self.clone();
                    board.move_piece(*unambiguous_move).unwrap();
                    is_match &= board.state().unwrap() == action.into();
                }
                is_match
            })
            .collect();
        match all_moves.len() {
            0 => Err(ChessError::ImpossibleMove(chess_move)),
            1 => Ok(all_moves[0]),
            _ => Err(ChessError::AmbiguousMove(chess_move)),
        }
    }

    fn disambiguate_castling(&self, chess_move: AmbiguousMove) -> SimpleMove {
        let side = match chess_move {
            AmbiguousMove::Normal { .. } => panic!("Can't use castling move disambiguator on normal move"),
            AmbiguousMove::Castle { side } => side,
        };
        let mut rank = 0;
        let mut file = 6;
        let mut src = SimpleSquare::new(4, 0);
        if side == CastlingSide::QueenSide {
            file = 2;
        }
        if self.turn == PieceColour::Black {
            rank = 7;
            src = SimpleSquare::new(4, 7);
        }

        SimpleMove::new(src, SimpleSquare::new(file, rank), None)
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

    fn moves_from_strs(moves: Vec<&str>) -> Vec<SimpleMove> {
        let mut new_moves: Vec<SimpleMove> = moves.iter().map(|s| SimpleMove::from_pgn_str(s).unwrap()).collect();
        new_moves.sort();
        new_moves
    }

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

    #[test]
    fn white_pawn_double_push() {
        let board = ChessBoard::starting_board();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("e2").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["e2e3", "e2e4"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn black_pawn_double_push() {
        let board = ChessBoard::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("h7").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["h7h6", "h7h5"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn pawn_double_push_blocked() {
        let board = ChessBoard::from_fen("rnbqkbnr/pppppppp/8/2P5/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("c7").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["c7c6"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn pawn_takes() {
        let board = ChessBoard::from_fen("rnbqkbnr/pppppppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2").unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("e4").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["e4e5", "e4d5"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn pawn_takes_en_passant_behind() {
        let board = ChessBoard::from_fen("rnbqkbnr/pppppppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e3 0 2").unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("e4").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["e4e5", "e4d5"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn en_passant() {
        let mut board = ChessBoard::from_fen("rnbqkbnr/pppppppp/8/5P2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2").unwrap();
        board.move_piece(SimpleMove::from_pgn_str("g7g5").unwrap()).unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("f5").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["f5f6", "f5g6"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn knight_moves() {
        let board = ChessBoard::from_fen("rnbqkbnr/ppppppp1/7p/8/6N1/8/PPPPPPPP/RNBQKB1R w KQkq - 0 2").unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("g4").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["g4h6", "g4f6", "g4e3", "g4e5"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn bishop_moves() {
        let board = ChessBoard::from_fen("rnbqkbnr/pppppppp/8/5b2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2").unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("f5").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["f5g6", "f5e6", "f5e4", "f5d3", "f5c2", "f5g4", "f5h3"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn rook_moves() {
        let board = ChessBoard::from_fen("rnbqkbnr/pppppppp/8/2R2P2/8/8/PPPPP1PP/RNBQKBNR w KQkq - 0 2").unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("c5").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["c5b5", "c5a5", "c5d5", "c5e5", "c5c6", "c5c7", "c5c4", "c5c3"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn queen_moves() {
        let board = ChessBoard::from_fen("rnbqkbnr/pppppppp/8/5Q2/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2").unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("f5").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec![
            "f5f6", "f5f7", "f5f4", "f5f3", "f5g5", "f5h5", "f5e5", "f5d5", "f5c5", "f5b5", "f5a5", "f5g6", "f5h7",
            "f5g4", "f5h3", "f5e6", "f5d7", "f5e4", "f5d3",
        ]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn king_moves() {
        let board = ChessBoard::from_fen("rnbqkbnr/pppp1ppp/4p3/4KP2/8/8/PPPP1PPP/RNBQ1BNR w KQkq - 0 2").unwrap();
        let mut moves: Vec<SimpleMove> = board
            .piece_plegal_moves(SimpleSquare::from_pgn_str("e5").unwrap())
            .unwrap()
            .into_iter()
            .collect();
        moves.sort();
        let exp_moves = moves_from_strs(vec!["e5e6", "e5f6", "e5d6", "e5d5", "e5d4", "e5e4", "e5f4"]);
        assert_eq!(moves, exp_moves)
    }

    #[test]
    fn king_in_check() {
        let board = ChessBoard::from_fen("k3r3/1P6/4K3/8/8/8/8/8 w KQkq - 0 2").unwrap();
        assert_eq!(board.king_in_check(PieceColour::White).unwrap(), true);
        assert_eq!(board.king_in_check(PieceColour::Black).unwrap(), true);
        assert_eq!(board.state().unwrap(), BoardState::Check);
    }

    #[test]
    fn king_not_in_check() {
        let board = ChessBoard::from_fen("k3r3/8/1P6/3K4/8/8/8/8 w KQkq - 0 2").unwrap();
        assert_eq!(board.king_in_check(PieceColour::White).unwrap(), false);
        assert_eq!(board.king_in_check(PieceColour::Black).unwrap(), false);
        assert_eq!(board.state().unwrap(), BoardState::Normal);
    }

    #[test]
    fn pinned_piece() {
        let board = ChessBoard::from_fen("k3r3/8/4N3/8/4K3/8/8/8 w KQkq - 0 2").unwrap();
        assert!(
            board
                .piece_legal_moves(SimpleSquare::from_pgn_str("e6").unwrap())
                .unwrap()
                .into_iter()
                .next()
                .is_none()
        )
    }
}
