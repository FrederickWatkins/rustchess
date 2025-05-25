use crate::types::{ChessMove, Position};
use crate::{error::*, piece::*, traits::*};
use std::fmt::Display;

type Directions = [Position; 8];

const BISHOP_DIRECTIONS: Directions = [
    Position(1, 1),
    Position(-1, 1),
    Position(-1, -1),
    Position(1, -1),
    Position(0, 0),
    Position(0, 0),
    Position(0, 0),
    Position(0, 0),
];

const ROOK_DIRECTIONS: Directions = [
    Position(0, 1),
    Position(1, 0),
    Position(-1, 0),
    Position(0, -1),
    Position(0, 0),
    Position(0, 0),
    Position(0, 0),
    Position(0, 0),
];

const QUEEN_DIRECTIONS: Directions = [
    Position(0, 1),
    Position(1, 0),
    Position(-1, 0),
    Position(0, -1),
    Position(1, 1),
    Position(-1, 1),
    Position(-1, -1),
    Position(1, -1),
];

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct CastlingRights {
    queen_side: bool,
    king_side: bool,
}

impl CastlingRights {
    fn new() -> Self {
        Self {
            queen_side: true,
            king_side: true,
        }
    }
}

/// Transparent board representation
///
/// Called "transparent" because the internal representation of the pieces is the same as the external representation,
/// just a vector of pieces and their colours and positions. The simplest to implement but also the least efficient.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct TransparentBoard {
    pieces: Vec<Piece>,
    turn: Colour,
    // The square that the en-passanting pawn can move to as used in FEN
    en_passant: Option<Position>,
    castling_rights: [CastlingRights; 2],
}

impl LegalMoveGenerator for TransparentBoard {
    fn check_king_safe(&self, chess_move: ChessMove) -> Result<bool, ChessError> {
        let mut test_board = self.clone();
        test_board.move_piece(chess_move).unwrap();
        if let Some(king) = test_board.get_piece_kind(PieceKind::King).iter().nth(0) {
            Ok(!test_board
                .all_plegal_moves()
                .iter()
                .any(|test_move| test_move.1 == king.pos))
        } else {
            Err(ChessError::NoKing)
        }
    }
}

impl PLegalMoveGenerator for TransparentBoard {
    fn all_plegal_moves(&self) -> Vec<ChessMove> {
        self.pieces
            .iter()
            .filter(|piece| piece.colour == self.turn)
            .map(|piece| self.piece_plegal_moves(piece.pos).unwrap())
            .flatten()
            .collect()
    }

    fn piece_plegal_moves(&self, pos: Position) -> Result<Vec<ChessMove>, ChessError> {
        if let Some(piece) = self.get_piece(pos) {
            if piece.colour != self.turn {
                return Err(ChessError::WrongColour(pos));
            }
            Ok(match piece.kind {
                PieceKind::Pawn => self.pawn_moves(piece),
                PieceKind::Knight => self.knight_moves(piece),
                PieceKind::King => self.king_moves(piece),
                _ => self.traversal_moves(piece),
            }
            // Keep all moves with destination square between 0 and 8
            .iter()
            .filter(|test_move| {
                (0..8).contains(&test_move.1 .0) && (0..8).contains(&test_move.1 .1)
            })
            .map(|chess_move| *chess_move)
            .collect())
        } else {
            Err(ChessError::PieceMissing(pos))
        }
    }

    fn check_move_plegal(&self, chess_move: ChessMove) -> Result<bool, ChessError> {
        Ok(self.piece_plegal_moves(chess_move.0)?.contains(&chess_move))
    }
}

impl Board for TransparentBoard {
    fn move_piece(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        if self.get_piece(chess_move.0).is_none() {
            Err(ChessError::PieceMissing(chess_move.0))
        } else {
            if let Some(taken_piece) = self
                .pieces
                .iter()
                .position(|piece| piece.pos == chess_move.1)
            {
                self.pieces.remove(taken_piece);
            }
            if let Some(piece) = self.get_piece_mut(chess_move.0) {
                piece.pos = chess_move.1;
            }
            self.turn = !self.turn;
            Ok(())
        }
    }

    fn from_fen(fen: &str) -> Result<Self, ChessError> {
        todo!()
    }
}

impl TransparentBoard {
    pub fn starting_board() -> Self {
        TransparentBoard {
            pieces: vec![
                Piece::new(Position(0, 0), Colour::White, PieceKind::Rook),
                Piece::new(Position(1, 0), Colour::White, PieceKind::Knight),
                Piece::new(Position(2, 0), Colour::White, PieceKind::Bishop),
                Piece::new(Position(3, 0), Colour::White, PieceKind::Queen),
                Piece::new(Position(4, 0), Colour::White, PieceKind::King),
                Piece::new(Position(5, 0), Colour::White, PieceKind::Bishop),
                Piece::new(Position(6, 0), Colour::White, PieceKind::Knight),
                Piece::new(Position(7, 0), Colour::White, PieceKind::Rook),
                Piece::new(Position(0, 1), Colour::White, PieceKind::Pawn),
                Piece::new(Position(1, 1), Colour::White, PieceKind::Pawn),
                Piece::new(Position(2, 1), Colour::White, PieceKind::Pawn),
                Piece::new(Position(3, 1), Colour::White, PieceKind::Pawn),
                Piece::new(Position(4, 1), Colour::White, PieceKind::Pawn),
                Piece::new(Position(5, 1), Colour::White, PieceKind::Pawn),
                Piece::new(Position(6, 1), Colour::White, PieceKind::Pawn),
                Piece::new(Position(7, 1), Colour::White, PieceKind::Pawn),
                Piece::new(Position(0, 7), Colour::Black, PieceKind::Rook),
                Piece::new(Position(1, 7), Colour::Black, PieceKind::Knight),
                Piece::new(Position(2, 7), Colour::Black, PieceKind::Bishop),
                Piece::new(Position(3, 7), Colour::Black, PieceKind::Queen),
                Piece::new(Position(4, 7), Colour::Black, PieceKind::King),
                Piece::new(Position(5, 7), Colour::Black, PieceKind::Bishop),
                Piece::new(Position(6, 7), Colour::Black, PieceKind::Knight),
                Piece::new(Position(7, 7), Colour::Black, PieceKind::Rook),
                Piece::new(Position(0, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(Position(1, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(Position(2, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(Position(3, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(Position(4, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(Position(5, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(Position(6, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(Position(7, 6), Colour::Black, PieceKind::Pawn),
            ],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        }
    }

    #[inline]
    fn get_all_pieces(&self) -> Vec<&Piece> {
        self.pieces.iter().collect()
    }

    #[inline]
    fn get_piece(&self, pos: Position) -> Option<&Piece> {
        self.pieces.iter().find(|&piece| piece.pos == pos)
    }

    #[inline]
    fn get_piece_kind(&self, kind: PieceKind) -> Vec<&Piece> {
        self.pieces
            .iter()
            .filter(|piece| piece.kind == kind)
            .collect()
    }

    #[inline]
    fn get_piece_mut(&mut self, pos: Position) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|piece| piece.pos == pos)
    }

    #[inline(always)] // Helper function for piece_plegal_moves so inline
    fn pawn_moves(&self, piece: &Piece) -> Vec<ChessMove> {
        let mut out: Vec<ChessMove> = vec![];
        // First square empty
        if self
            .get_piece(piece.pos + piece.direction(Position(0, 1)))
            .is_none()
        {
            out.push(ChessMove(
                piece.pos,
                piece.pos + piece.direction(Position(0, 1)),
            ));
            // Piece on starting row and second square empty
            if self
                .get_piece(piece.pos + piece.direction(Position(0, 2)))
                .is_none()
                && piece.pos.1
                    == match piece.colour {
                        Colour::White => 1,
                        Colour::Black => 6,
                    }
            {
                out.push(ChessMove(
                    piece.pos,
                    piece.pos + piece.direction(Position(0, 2)),
                ))
            }
        }
        if let Some(other_piece) = self.get_piece(piece.pos + piece.direction(Position(1, 1))) {
            if other_piece.colour != piece.colour {
                out.push(ChessMove(
                    piece.pos,
                    piece.pos + piece.direction(Position(1, 1)),
                ));
            }
        }
        if let Some(other_piece) = self.get_piece(piece.pos + piece.direction(Position(-1, 1))) {
            if other_piece.colour != piece.colour {
                out.push(ChessMove(
                    piece.pos,
                    piece.pos + piece.direction(Position(-1, 1)),
                ));
            }
        }
        if let Some(en_passant) = self.en_passant {
            if en_passant == piece.pos + piece.direction(Position(1, 1))
                || en_passant == piece.pos + piece.direction(Position(-1, 1))
            {
                out.push(ChessMove(piece.pos, en_passant));
            }
        }
        out
    }

    #[inline(always)] // Helper function for piece_plegal_moves so inline
    fn knight_moves(&self, piece: &Piece) -> Vec<ChessMove> {
        let mut out: Vec<ChessMove> = vec![];
        let knight_directions = [
            Position(1, 2),
            Position(2, 1),
            Position(-1, 2),
            Position(-2, 1),
            Position(-1, -2),
            Position(-2, -1),
            Position(1, -2),
            Position(2, -1),
        ];
        for direction in knight_directions {
            if self.check_square_takeable(piece, piece.pos + direction) {
                out.push(ChessMove(piece.pos, piece.pos + direction))
            }
        }
        out
    }

    #[inline(always)] // Helper function for piece_plegal_moves so inline
    fn traversal_moves(&self, piece: &Piece) -> Vec<ChessMove> {
        let mut out: Vec<ChessMove> = vec![];
        let directions = match piece.kind {
            PieceKind::Bishop => BISHOP_DIRECTIONS,
            PieceKind::Rook => ROOK_DIRECTIONS,
            PieceKind::Queen => QUEEN_DIRECTIONS,
            other => panic!("{:?} is not a traversal piece", other),
        };

        for direction in directions {
            let mut curr_pos = piece.pos + direction;
            while self.get_piece(curr_pos).is_none()
                && (0..8).contains(&curr_pos.0)
                && (0..8).contains(&curr_pos.1)
            {
                out.push(ChessMove(piece.pos, curr_pos));
                curr_pos += direction;
            }
            if self.check_square_takeable(piece, curr_pos) {
                out.push(ChessMove(piece.pos, curr_pos));
            }
        }
        out
    }

    #[inline(always)] // Helper function for piece_plegal_moves so inline
    fn king_moves(&self, piece: &Piece) -> Vec<ChessMove> {
        let mut out: Vec<ChessMove> = vec![];
        for i in -1..=1 {
            for j in -1..=1 {
                let pos = piece.pos + Position(i, j);
                if self.check_square_takeable(piece, pos) {
                    out.push(ChessMove(piece.pos, pos))
                }
            }
        }
        out
    }

    #[inline]
    fn check_square_takeable(&self, piece: &Piece, square: Position) -> bool {
        if let Some(other_piece) = self.get_piece(square) {
            other_piece.colour != piece.colour
        } else {
            true
        }
    }

    fn fmt_board(&self) -> String {
        let templ = "                       \n";
        let mut outstr = String::from(templ).repeat(8);
        for piece in &self.pieces {
            let index = (7 - piece.pos.1 as usize) * templ.len() + piece.pos.0 as usize * 3;
            outstr.replace_range(index..index + 2, &format!("{:2}", <&str>::from(piece.kind)));
        }
        outstr
    }
}

impl Display for TransparentBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.fmt_board())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_white_pawn_moves() {
        let pawn_pos = Position(3, 1);
        let ourpawn = Piece::new(pawn_pos, Colour::White, PieceKind::Pawn);
        let capturepawn = Piece::new(Position(2, 2), Colour::Black, PieceKind::Pawn);
        let blockingpiece = Piece::new(Position(3, 3), Colour::White, PieceKind::Knight);
        let mut board = TransparentBoard {
            pieces: vec![ourpawn, capturepawn],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(Position(3, 1)).unwrap();
        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&ChessMove(pawn_pos, Position(3, 2))));
        assert!(moves.contains(&ChessMove(pawn_pos, Position(3, 3))));
        assert!(moves.contains(&ChessMove(pawn_pos, Position(2, 2))));
        board.pieces.push(blockingpiece);
        board.get_piece_mut(Position(2, 2)).unwrap().pos = Position(4, 2);
        moves = board.piece_plegal_moves(Position(3, 1)).unwrap();
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&ChessMove(pawn_pos, Position(3, 2))));
        assert!(moves.contains(&ChessMove(pawn_pos, Position(4, 2))));
        board.get_piece_mut(Position(3, 3)).unwrap().pos = Position(3, 2);
        moves = board.piece_plegal_moves(Position(3, 1)).unwrap();
        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&ChessMove(pawn_pos, Position(4, 2))));
    }

    #[test]
    fn test_black_pawn_moves() {
        let pawn_pos = Position(3, 3);
        let ourpawn = Piece::new(pawn_pos, Colour::Black, PieceKind::Pawn);
        let capturepawn = Piece::new(Position(2, 2), Colour::White, PieceKind::Pawn);
        let blockingpiece = Piece::new(Position(3, 2), Colour::White, PieceKind::Knight);
        let mut board = TransparentBoard {
            pieces: vec![ourpawn, capturepawn],
            turn: Colour::Black,
            en_passant: Some(Position(4, 2)),
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(Position(3, 3)).unwrap();
        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&ChessMove(pawn_pos, Position(3, 2))));
        assert!(moves.contains(&ChessMove(pawn_pos, Position(4, 2))));
        assert!(moves.contains(&ChessMove(pawn_pos, Position(2, 2))));
        board.pieces.push(blockingpiece);
        moves = board.piece_plegal_moves(Position(3, 3)).unwrap();
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&ChessMove(pawn_pos, Position(2, 2))));
        assert!(moves.contains(&ChessMove(pawn_pos, Position(4, 2))));
    }

    #[test]
    fn test_knight_moves() {
        let knight_pos = Position(3, 1);
        let ourknight = Piece::new(knight_pos, Colour::Black, PieceKind::Knight);
        let capturepawn = Piece::new(Position(4, 3), Colour::White, PieceKind::Pawn);
        let blockingpiece = Piece::new(Position(1, 2), Colour::Black, PieceKind::Knight);
        let board = TransparentBoard {
            pieces: vec![ourknight, capturepawn, blockingpiece],
            turn: Colour::Black,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(Position(3, 1)).unwrap();
        let mut expectation = vec![
            ChessMove(knight_pos, Position(4, 3)),
            ChessMove(knight_pos, Position(2, 3)),
            ChessMove(knight_pos, Position(5, 2)),
            ChessMove(knight_pos, Position(5, 0)),
            ChessMove(knight_pos, Position(1, 0)),
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_bishop_moves() {
        let bishop_pos = Position(4, 3);
        let ourbishop = Piece::new(bishop_pos, Colour::Black, PieceKind::Bishop);
        let captureknight = Piece::new(Position(7, 6), Colour::White, PieceKind::Knight);
        let blockingpiece = Piece::new(Position(6, 1), Colour::Black, PieceKind::King);
        let board = TransparentBoard {
            pieces: vec![ourbishop, captureknight, blockingpiece],
            turn: Colour::Black,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(Position(4, 3)).unwrap();
        let mut expectation = vec![
            ChessMove(bishop_pos, Position(5, 4)),
            ChessMove(bishop_pos, Position(6, 5)),
            ChessMove(bishop_pos, Position(7, 6)),
            ChessMove(bishop_pos, Position(3, 2)),
            ChessMove(bishop_pos, Position(2, 1)),
            ChessMove(bishop_pos, Position(1, 0)),
            ChessMove(bishop_pos, Position(5, 2)),
            ChessMove(bishop_pos, Position(3, 4)),
            ChessMove(bishop_pos, Position(2, 5)),
            ChessMove(bishop_pos, Position(1, 6)),
            ChessMove(bishop_pos, Position(0, 7)),
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_rook_moves() {
        let rook_pos = Position(4, 3);
        let ourrook = Piece::new(rook_pos, Colour::White, PieceKind::Rook);
        let captureknight = Piece::new(Position(4, 6), Colour::Black, PieceKind::Knight);
        let blockingpiece = Piece::new(Position(6, 3), Colour::White, PieceKind::King);
        let board = TransparentBoard {
            pieces: vec![ourrook, captureknight, blockingpiece],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(Position(4, 3)).unwrap();
        let mut expectation = vec![
            ChessMove(rook_pos, Position(0, 3)),
            ChessMove(rook_pos, Position(1, 3)),
            ChessMove(rook_pos, Position(2, 3)),
            ChessMove(rook_pos, Position(3, 3)),
            ChessMove(rook_pos, Position(5, 3)),
            ChessMove(rook_pos, Position(4, 0)),
            ChessMove(rook_pos, Position(4, 1)),
            ChessMove(rook_pos, Position(4, 2)),
            ChessMove(rook_pos, Position(4, 4)),
            ChessMove(rook_pos, Position(4, 5)),
            ChessMove(rook_pos, Position(4, 6)),
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_queen_moves() {
        let queen_pos = Position(4, 3);
        let ourqueen = Piece::new(queen_pos, Colour::White, PieceKind::Queen);
        let captureknight = Piece::new(Position(4, 6), Colour::Black, PieceKind::Knight);
        let blockingpiece = Piece::new(Position(6, 3), Colour::White, PieceKind::King);
        let capturepawn = Piece::new(Position(7, 6), Colour::Black, PieceKind::Pawn);
        let blockingpiece2 = Piece::new(Position(6, 1), Colour::White, PieceKind::Queen);
        let board = TransparentBoard {
            pieces: vec![
                ourqueen,
                captureknight,
                blockingpiece,
                capturepawn,
                blockingpiece2,
            ],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(Position(4, 3)).unwrap();
        let mut expectation = vec![
            ChessMove(queen_pos, Position(0, 3)),
            ChessMove(queen_pos, Position(1, 3)),
            ChessMove(queen_pos, Position(2, 3)),
            ChessMove(queen_pos, Position(3, 3)),
            ChessMove(queen_pos, Position(5, 3)),
            ChessMove(queen_pos, Position(4, 0)),
            ChessMove(queen_pos, Position(4, 1)),
            ChessMove(queen_pos, Position(4, 2)),
            ChessMove(queen_pos, Position(4, 4)),
            ChessMove(queen_pos, Position(4, 5)),
            ChessMove(queen_pos, Position(4, 6)),
            ChessMove(queen_pos, Position(5, 4)),
            ChessMove(queen_pos, Position(6, 5)),
            ChessMove(queen_pos, Position(7, 6)),
            ChessMove(queen_pos, Position(3, 2)),
            ChessMove(queen_pos, Position(2, 1)),
            ChessMove(queen_pos, Position(1, 0)),
            ChessMove(queen_pos, Position(5, 2)),
            ChessMove(queen_pos, Position(3, 4)),
            ChessMove(queen_pos, Position(2, 5)),
            ChessMove(queen_pos, Position(1, 6)),
            ChessMove(queen_pos, Position(0, 7)),
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_king_moves() {
        let king_pos = Position(4, 3);
        let ourking = Piece::new(Position(4, 3), Colour::White, PieceKind::King);
        let coveringrook = Piece::new(Position(3, 6), Colour::Black, PieceKind::Rook);
        let coveringqueen = Piece::new(Position(5, 7), Colour::Black, PieceKind::Queen);
        let capturepawn = Piece::new(Position(4, 4), Colour::Black, PieceKind::Pawn);
        let blockingpiece2 = Piece::new(Position(4, 2), Colour::White, PieceKind::Queen);
        let blockingpiece3 = Piece::new(Position(5, 3), Colour::Black, PieceKind::Knight);
        let board = TransparentBoard {
            pieces: vec![
                ourking,
                coveringrook,
                coveringqueen,
                capturepawn,
                blockingpiece2,
                blockingpiece3,
            ],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [
                CastlingRights {
                    queen_side: false,
                    king_side: false,
                },
                CastlingRights {
                    queen_side: false,
                    king_side: false,
                },
            ],
        };
        assert_eq!(
            board.get_piece_kind(PieceKind::King)[0],
            board.get_piece(king_pos).unwrap()
        );
        let mut moves = board.piece_legal_moves(Position(4, 3)).unwrap();
        let mut expectation = vec![
            ChessMove(king_pos, Position(4, 4)),
            ChessMove(king_pos, Position(5, 2)),
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }
}
