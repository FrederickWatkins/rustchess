use crate::types::{i8_to_file, i8_to_rank, ChessMove, IntChessSquare};
use crate::{error::*, piece::*, traits::*};
use std::fmt::Display;

type Directions = [IntChessSquare; 8];

const BISHOP_DIRECTIONS: Directions = [
    IntChessSquare(1, 1),
    IntChessSquare(-1, 1),
    IntChessSquare(-1, -1),
    IntChessSquare(1, -1),
    IntChessSquare(0, 0),
    IntChessSquare(0, 0),
    IntChessSquare(0, 0),
    IntChessSquare(0, 0),
];

const ROOK_DIRECTIONS: Directions = [
    IntChessSquare(0, 1),
    IntChessSquare(1, 0),
    IntChessSquare(-1, 0),
    IntChessSquare(0, -1),
    IntChessSquare(0, 0),
    IntChessSquare(0, 0),
    IntChessSquare(0, 0),
    IntChessSquare(0, 0),
];

const QUEEN_DIRECTIONS: Directions = [
    IntChessSquare(0, 1),
    IntChessSquare(1, 0),
    IntChessSquare(-1, 0),
    IntChessSquare(0, -1),
    IntChessSquare(1, 1),
    IntChessSquare(-1, 1),
    IntChessSquare(-1, -1),
    IntChessSquare(1, -1),
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
    en_passant: Option<IntChessSquare>,
    castling_rights: [CastlingRights; 2],
}

impl LegalMoveGenerator for TransparentBoard {
    fn check_king_safe(&self, colour: Colour) -> bool {
        let mut temp_board = self.clone(); // TODO: Fix this slow clone
        temp_board.turn = !colour;
        !temp_board
            .get_piece_kind(PieceKind::King)
            .into_iter()
            .filter(|king| king.colour == colour)
            .any(|king| {
                temp_board
                    .all_plegal_moves()
                    .into_iter()
                    .any(|test_move| test_move.end == king.pos)
            })
    }
}

impl PLegalMoveGenerator for TransparentBoard {
    fn all_plegal_moves(&self) -> Vec<ChessMove> {
        self.pieces
            .iter()
            .filter(|piece| piece.colour == self.turn)
            .flat_map(|piece| self.piece_plegal_moves(piece.pos).unwrap())
            .collect()
    }

    fn piece_plegal_moves(&self, pos: IntChessSquare) -> Result<Vec<ChessMove>, ChessError> {
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
            .into_iter()
            .filter(|test_move| {
                (0..8).contains(&test_move.end.0) && (0..8).contains(&test_move.end.1)
            })
            .collect())
        } else {
            Err(ChessError::PieceMissing(pos))
        }
    }

    fn check_move_plegal(&self, chess_move: ChessMove) -> Result<bool, ChessError> {
        Ok(self
            .piece_plegal_moves(chess_move.start)?
            .contains(&chess_move))
    }
}

impl Board for TransparentBoard {
    fn move_piece(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        if self.get_piece(chess_move.start).is_none() {
            Err(ChessError::PieceMissing(chess_move.start))
        } else {
            if let Some(taken_piece) = self
                .pieces
                .iter()
                .position(|piece| piece.pos == chess_move.end)
            {
                self.pieces.remove(taken_piece);
            }
            let piece = self.get_piece_mut(chess_move.start).unwrap();
            let kind = piece.kind;
            piece.pos = chess_move.end;

            if let Some(promote) = chess_move.promote {
                piece.kind = promote;
            }

            if piece.kind == PieceKind::King && chess_move.start.0 == 4 {
                match chess_move.end.0 {
                    1 => {
                        self.get_piece_mut(IntChessSquare(0, chess_move.start.1))
                            .ok_or(ChessError::IllegalMove(chess_move))?
                            .pos = IntChessSquare(2, chess_move.start.1)
                    }
                    6 => {
                        self.get_piece_mut(IntChessSquare(7, chess_move.start.1))
                            .ok_or(ChessError::IllegalMove(chess_move))?
                            .pos = IntChessSquare(5, chess_move.start.1)
                    }
                    _ => (),
                }
            }
            if kind == PieceKind::Pawn
                && chess_move.end == chess_move.start + self.turn.direction(IntChessSquare(0, 2))
            {
                self.en_passant = Some(chess_move.start + self.turn.direction(IntChessSquare(0, 1)));
            } else {
                self.en_passant = None;
            }
            if kind == PieceKind::Pawn
                && chess_move.end == self.en_passant.unwrap_or(IntChessSquare(0, 0))
            {
                if let Some(taken_piece) = self.pieces.iter().position(|other_piece| {
                    other_piece.pos == chess_move.end + other_piece.colour.direction(IntChessSquare(0, 1))
                }) {
                    self.pieces.remove(taken_piece);
                }
            }
            self.turn = !self.turn;
            Ok(())
        }
    }

    #[inline]
    fn get_piece(&self, pos: IntChessSquare) -> Option<&Piece> {
        self.pieces.iter().find(|&piece| piece.pos == pos)
    }

    fn get_all_pieces(&self) -> &Vec<Piece> {
        &self.pieces
    }

    fn from_fen(fen: &str) -> Result<Self, ChessError> {
        todo!()
    }

    fn turn(&self) -> Colour {
        self.turn
    }

    fn starting_board() -> Self {
        TransparentBoard {
            pieces: vec![
                Piece::new(IntChessSquare(0, 0), Colour::White, PieceKind::Rook),
                Piece::new(IntChessSquare(1, 0), Colour::White, PieceKind::Knight),
                Piece::new(IntChessSquare(2, 0), Colour::White, PieceKind::Bishop),
                Piece::new(IntChessSquare(3, 0), Colour::White, PieceKind::Queen),
                Piece::new(IntChessSquare(4, 0), Colour::White, PieceKind::King),
                Piece::new(IntChessSquare(5, 0), Colour::White, PieceKind::Bishop),
                Piece::new(IntChessSquare(6, 0), Colour::White, PieceKind::Knight),
                Piece::new(IntChessSquare(7, 0), Colour::White, PieceKind::Rook),
                Piece::new(IntChessSquare(0, 1), Colour::White, PieceKind::Pawn),
                Piece::new(IntChessSquare(1, 1), Colour::White, PieceKind::Pawn),
                Piece::new(IntChessSquare(2, 1), Colour::White, PieceKind::Pawn),
                Piece::new(IntChessSquare(3, 1), Colour::White, PieceKind::Pawn),
                Piece::new(IntChessSquare(4, 1), Colour::White, PieceKind::Pawn),
                Piece::new(IntChessSquare(5, 1), Colour::White, PieceKind::Pawn),
                Piece::new(IntChessSquare(6, 1), Colour::White, PieceKind::Pawn),
                Piece::new(IntChessSquare(7, 1), Colour::White, PieceKind::Pawn),
                Piece::new(IntChessSquare(0, 7), Colour::Black, PieceKind::Rook),
                Piece::new(IntChessSquare(1, 7), Colour::Black, PieceKind::Knight),
                Piece::new(IntChessSquare(2, 7), Colour::Black, PieceKind::Bishop),
                Piece::new(IntChessSquare(3, 7), Colour::Black, PieceKind::Queen),
                Piece::new(IntChessSquare(4, 7), Colour::Black, PieceKind::King),
                Piece::new(IntChessSquare(5, 7), Colour::Black, PieceKind::Bishop),
                Piece::new(IntChessSquare(6, 7), Colour::Black, PieceKind::Knight),
                Piece::new(IntChessSquare(7, 7), Colour::Black, PieceKind::Rook),
                Piece::new(IntChessSquare(0, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(IntChessSquare(1, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(IntChessSquare(2, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(IntChessSquare(3, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(IntChessSquare(4, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(IntChessSquare(5, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(IntChessSquare(6, 6), Colour::Black, PieceKind::Pawn),
                Piece::new(IntChessSquare(7, 6), Colour::Black, PieceKind::Pawn),
            ],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        }
    }
}

impl TransparentBoard {
    #[inline]
    fn get_all_pieces(&self) -> Vec<&Piece> {
        self.pieces.iter().collect()
    }

    #[inline]
    fn get_piece_kind(&self, kind: PieceKind) -> Vec<&Piece> {
        self.pieces
            .iter()
            .filter(|piece| piece.kind == kind)
            .collect()
    }

    #[inline]
    fn get_piece_mut(&mut self, pos: IntChessSquare) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|piece| piece.pos == pos)
    }

    #[inline(always)] // Helper function for piece_plegal_moves so inline
    fn pawn_moves(&self, piece: &Piece) -> Vec<ChessMove> {
        let mut out: Vec<ChessMove> = vec![];
        // First square empty
        if self
            .get_piece(piece.pos + piece.direction(IntChessSquare(0, 1)))
            .is_none()
        {
            out.append(
                &mut self.pawn_promotions(piece.pos, piece.pos + piece.direction(IntChessSquare(0, 1))),
            );
            // Piece on starting row and second square empty
            if self
                .get_piece(piece.pos + piece.direction(IntChessSquare(0, 2)))
                .is_none()
                && piece.pos.1
                    == match piece.colour {
                        Colour::White => 1,
                        Colour::Black => 6,
                    }
            {
                out.push(ChessMove {
                    start: piece.pos,
                    end: piece.pos + piece.direction(IntChessSquare(0, 2)),
                    promote: None,
                })
            }
        }
        if let Some(other_piece) = self.get_piece(piece.pos + piece.direction(IntChessSquare(1, 1))) {
            if other_piece.colour != piece.colour {
                out.append(
                    &mut self
                        .pawn_promotions(piece.pos, piece.pos + piece.direction(IntChessSquare(1, 1))),
                );
            }
        }
        if let Some(other_piece) = self.get_piece(piece.pos + piece.direction(IntChessSquare(-1, 1))) {
            if other_piece.colour != piece.colour {
                
            out.append(&mut self.pawn_promotions(piece.pos, piece.pos + piece.direction(IntChessSquare(-1, 1))));
            }
        }
        if let Some(en_passant) = self.en_passant {
            if en_passant == piece.pos + piece.direction(IntChessSquare(1, 1))
                || en_passant == piece.pos + piece.direction(IntChessSquare(-1, 1))
            {
                out.push(ChessMove {
                    start: piece.pos,
                    end: en_passant,
                    promote: None,
                });
            }
        }
        out
    }

    fn pawn_promotions(&self, start: IntChessSquare, end: IntChessSquare) -> Vec<ChessMove> {
        if end.1 == 0 || end.1 == 7 {
            vec![
                ChessMove {
                    start,
                    end,
                    promote: Some(PieceKind::Knight),
                },
                ChessMove {
                    start,
                    end,
                    promote: Some(PieceKind::Bishop),
                },
                ChessMove {
                    start,
                    end,
                    promote: Some(PieceKind::Rook),
                },
                ChessMove {
                    start,
                    end,
                    promote: Some(PieceKind::Queen),
                },
            ]
        } else {
            vec![ChessMove {
                start,
                end,
                promote: None,
            }]
        }
    }

    #[inline(always)] // Helper function for piece_plegal_moves so inline
    fn knight_moves(&self, piece: &Piece) -> Vec<ChessMove> {
        let mut out: Vec<ChessMove> = vec![];
        let knight_directions = [
            IntChessSquare(1, 2),
            IntChessSquare(2, 1),
            IntChessSquare(-1, 2),
            IntChessSquare(-2, 1),
            IntChessSquare(-1, -2),
            IntChessSquare(-2, -1),
            IntChessSquare(1, -2),
            IntChessSquare(2, -1),
        ];
        for direction in knight_directions {
            if self.check_square_takeable(piece, piece.pos + direction) {
                out.push(ChessMove {
                    start: piece.pos,
                    end: piece.pos + direction,
                    promote: None,
                })
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
                out.push(ChessMove {
                    start: piece.pos,
                    end: curr_pos,
                    promote: None,
                });
                curr_pos += direction;
            }
            if self.check_square_takeable(piece, curr_pos) {
                out.push(ChessMove {
                    start: piece.pos,
                    end: curr_pos,
                    promote: None,
                });
            }
        }
        out
    }

    #[inline(always)] // Helper function for piece_plegal_moves so inline
    fn king_moves(&self, piece: &Piece) -> Vec<ChessMove> {
        let mut out: Vec<ChessMove> = vec![];
        for i in -1..=1 {
            for j in -1..=1 {
                let pos = piece.pos + IntChessSquare(i, j);
                if self.check_square_takeable(piece, pos) {
                    out.push(ChessMove {
                        start: piece.pos,
                        end: pos,
                        promote: None,
                    })
                }
            }
        }
        out
    }

    #[inline]
    fn check_square_takeable(&self, piece: &Piece, square: IntChessSquare) -> bool {
        if let Some(other_piece) = self.get_piece(square) {
            other_piece.colour != piece.colour
        } else {
            true
        }
    }

    fn fmt_board(&self) -> String {
        let mut outstr = String::with_capacity(172);
        for i in (0..8).rev() {
            outstr.push(i8_to_rank(i));
            for j in 0..8 {
                outstr.push(' ');
                if let Some(piece) = self.get_piece(IntChessSquare(j, i)) {
                    outstr.push(char::from(*piece));
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
            outstr.push(i8_to_file(j));
            outstr.push(' ');
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
    // TODO add tests for pawn promotion and for move_piece_checked
    use crate::types::AmbiguousMove;

    use super::*;

    #[test]
    fn test_first_white_pawn_moves() {
        let pawn_pos = IntChessSquare(3, 1);
        let ourpawn = Piece::new(pawn_pos, Colour::White, PieceKind::Pawn);
        let capturepawn = Piece::new(IntChessSquare(2, 2), Colour::Black, PieceKind::Pawn);
        let blockingpiece = Piece::new(IntChessSquare(3, 3), Colour::White, PieceKind::Knight);
        let mut board = TransparentBoard {
            pieces: vec![ourpawn, capturepawn],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(IntChessSquare(3, 1)).unwrap();
        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(3, 2),
            promote: None
        }));
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(3, 3),
            promote: None
        }));
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(2, 2),
            promote: None
        }));
        board.pieces.push(blockingpiece);
        board.get_piece_mut(IntChessSquare(2, 2)).unwrap().pos = IntChessSquare(4, 2);
        moves = board.piece_plegal_moves(IntChessSquare(3, 1)).unwrap();
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(3, 2),
            promote: None
        }));
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(4, 2),
            promote: None
        }));
        board.get_piece_mut(IntChessSquare(3, 3)).unwrap().pos = IntChessSquare(3, 2);
        moves = board.piece_plegal_moves(IntChessSquare(3, 1)).unwrap();
        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(4, 2),
            promote: None
        }));
    }

    #[test]
    fn test_black_pawn_moves() {
        let pawn_pos = IntChessSquare(3, 3);
        let ourpawn = Piece::new(pawn_pos, Colour::Black, PieceKind::Pawn);
        let capturepawn = Piece::new(IntChessSquare(2, 2), Colour::White, PieceKind::Pawn);
        let blockingpiece = Piece::new(IntChessSquare(3, 2), Colour::White, PieceKind::Knight);
        let mut board = TransparentBoard {
            pieces: vec![ourpawn, capturepawn],
            turn: Colour::Black,
            en_passant: Some(IntChessSquare(4, 2)),
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(IntChessSquare(3, 3)).unwrap();
        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(3, 2),
            promote: None
        }));
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(4, 2),
            promote: None
        }));
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(2, 2),
            promote: None
        }));
        board.pieces.push(blockingpiece);
        moves = board.piece_plegal_moves(IntChessSquare(3, 3)).unwrap();
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(2, 2),
            promote: None
        }));
        assert!(moves.contains(&ChessMove {
            start: pawn_pos,
            end: IntChessSquare(4, 2),
            promote: None
        }));
    }

    #[test]
    fn test_knight_moves() {
        let knight_pos = IntChessSquare(3, 1);
        let ourknight = Piece::new(knight_pos, Colour::Black, PieceKind::Knight);
        let capturepawn = Piece::new(IntChessSquare(4, 3), Colour::White, PieceKind::Pawn);
        let blockingpiece = Piece::new(IntChessSquare(1, 2), Colour::Black, PieceKind::Knight);
        let board = TransparentBoard {
            pieces: vec![ourknight, capturepawn, blockingpiece],
            turn: Colour::Black,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(IntChessSquare(3, 1)).unwrap();
        let mut expectation = vec![
            ChessMove {
                start: knight_pos,
                end: IntChessSquare(4, 3),
                promote: None,
            },
            ChessMove {
                start: knight_pos,
                end: IntChessSquare(2, 3),
                promote: None,
            },
            ChessMove {
                start: knight_pos,
                end: IntChessSquare(5, 2),
                promote: None,
            },
            ChessMove {
                start: knight_pos,
                end: IntChessSquare(5, 0),
                promote: None,
            },
            ChessMove {
                start: knight_pos,
                end: IntChessSquare(1, 0),
                promote: None,
            },
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_bishop_moves() {
        let bishop_pos = IntChessSquare(4, 3);
        let ourbishop = Piece::new(bishop_pos, Colour::Black, PieceKind::Bishop);
        let captureknight = Piece::new(IntChessSquare(7, 6), Colour::White, PieceKind::Knight);
        let blockingpiece = Piece::new(IntChessSquare(6, 1), Colour::Black, PieceKind::King);
        let board = TransparentBoard {
            pieces: vec![ourbishop, captureknight, blockingpiece],
            turn: Colour::Black,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(IntChessSquare(4, 3)).unwrap();
        let mut expectation = vec![
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(5, 4),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(6, 5),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(7, 6),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(3, 2),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(2, 1),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(1, 0),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(5, 2),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(3, 4),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(2, 5),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(1, 6),
                promote: None,
            },
            ChessMove {
                start: bishop_pos,
                end: IntChessSquare(0, 7),
                promote: None,
            },
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_rook_moves() {
        let rook_pos = IntChessSquare(4, 3);
        let ourrook = Piece::new(rook_pos, Colour::White, PieceKind::Rook);
        let captureknight = Piece::new(IntChessSquare(4, 6), Colour::Black, PieceKind::Knight);
        let blockingpiece = Piece::new(IntChessSquare(6, 3), Colour::White, PieceKind::King);
        let board = TransparentBoard {
            pieces: vec![ourrook, captureknight, blockingpiece],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.piece_plegal_moves(IntChessSquare(4, 3)).unwrap();
        let mut expectation = vec![
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(0, 3),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(1, 3),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(2, 3),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(3, 3),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(5, 3),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(4, 0),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(4, 1),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(4, 2),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(4, 4),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(4, 5),
                promote: None,
            },
            ChessMove {
                start: rook_pos,
                end: IntChessSquare(4, 6),
                promote: None,
            },
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_queen_moves() {
        let queen_pos = IntChessSquare(4, 3);
        let ourqueen = Piece::new(queen_pos, Colour::White, PieceKind::Queen);
        let captureknight = Piece::new(IntChessSquare(4, 6), Colour::Black, PieceKind::Knight);
        let blockingpiece = Piece::new(IntChessSquare(6, 3), Colour::White, PieceKind::King);
        let capturepawn = Piece::new(IntChessSquare(7, 6), Colour::Black, PieceKind::Pawn);
        let blockingpiece2 = Piece::new(IntChessSquare(6, 1), Colour::White, PieceKind::Queen);
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
        let mut moves = board.piece_plegal_moves(IntChessSquare(4, 3)).unwrap();
        let mut expectation = vec![
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(0, 3),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(1, 3),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(2, 3),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(3, 3),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(5, 3),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(4, 0),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(4, 1),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(4, 2),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(4, 4),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(4, 5),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(4, 6),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(5, 4),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(6, 5),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(7, 6),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(3, 2),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(2, 1),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(1, 0),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(5, 2),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(3, 4),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(2, 5),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(1, 6),
                promote: None,
            },
            ChessMove {
                start: queen_pos,
                end: IntChessSquare(0, 7),
                promote: None,
            },
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_king_moves() {
        let king_pos = IntChessSquare(4, 3);
        let ourking = Piece::new(IntChessSquare(4, 3), Colour::White, PieceKind::King);
        let coveringrook = Piece::new(IntChessSquare(3, 6), Colour::Black, PieceKind::Rook);
        let coveringqueen = Piece::new(IntChessSquare(5, 7), Colour::Black, PieceKind::Queen);
        let capturepawn = Piece::new(IntChessSquare(4, 4), Colour::Black, PieceKind::Pawn);
        let blockingpiece2 = Piece::new(IntChessSquare(4, 2), Colour::White, PieceKind::Queen);
        let blockingpiece3 = Piece::new(IntChessSquare(5, 3), Colour::Black, PieceKind::Knight);
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
        let mut moves = board.piece_legal_moves(IntChessSquare(4, 3)).unwrap();
        let mut expectation = vec![
            ChessMove {
                start: king_pos,
                end: IntChessSquare(4, 4),
                promote: None,
            },
            ChessMove {
                start: king_pos,
                end: IntChessSquare(5, 2),
                promote: None,
            },
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_move_disamb() {
        let amb_move = AmbiguousMove::Standard {
            end: IntChessSquare(4, 3),
            kind: PieceKind::Pawn,
            start_file: None,
            start_rank: None,
            promote: None,
        };
        let board = TransparentBoard::starting_board();
        println!("{:?}", board.all_legal_moves());
        assert_eq!(
            board.disambiguate_move(amb_move).unwrap(),
            ChessMove {
                start: IntChessSquare(4, 1),
                end: IntChessSquare(4, 3),
                promote: None
            }
        );
    }
}
