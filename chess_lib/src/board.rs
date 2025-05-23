use std::fmt::Display;

use crate::chess_move::*;
use crate::piece::*;

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Board {
    pieces: Vec<Piece>,
    turn: Colour,
    // The square that the en-passanting pawn can move to as used in FEN
    en_passant: Option<Position>,
    castling_rights: [CastlingRights; 2],
}

impl Board {
    fn get_all_pieces(&self) -> Vec<&Piece> {
        self.pieces.iter().collect()
    }

    fn get_piece(&self, pos: Position) -> Option<&Piece> {
        self.pieces.iter().find(|&piece| piece.pos == pos)
    }

    fn get_piece_mut(&mut self, pos: Position) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|piece| piece.pos == pos)
    }

    fn check_move(&self, start: Position, end: Position) -> Result<bool, ()> {
        Ok(self.get_piece_moves(start)?.contains(&end))
    }

    fn get_all_moves(&self) -> Vec<UnambiguousMove> {
        let mut out: Vec<UnambiguousMove> = vec![];
        for piece in self.pieces.iter().filter(|piece| piece.colour == self.turn) {
            for piece_move in self.get_piece_moves(piece.pos).unwrap() {
                out.push(UnambiguousMove::new(piece.pos, piece_move))
            }
        }
        out
    }

    fn get_piece_moves(&self, pos: Position) -> Result<Vec<Position>, ()> {
        if let Some(piece) = self.get_piece(pos) {
            if piece.colour != self.turn {
                return Err(());
            }
            let mut out = match piece.kind {
                PieceKind::Pawn => self.pawn_moves(piece),
                PieceKind::Knight => self.knight_moves(piece),
                PieceKind::King => self.king_moves(piece),
                _ => self.traversal_moves(piece),
            };
            out.retain(|pos| (0..8).contains(&pos.0) && (0..8).contains(&pos.1));
            Ok(out)
        } else {
            Err(())
        }
    }

    fn move_piece(&mut self, start: Position, end: Position) -> Result<(), ()> {
        if self.check_move(start, end)? {
            self.move_piece_unchecked(start, end);
            Ok(())
        } else {
            Err(())
        }
    }

    fn move_piece_unchecked(&mut self, start: Position, end: Position) {
        if self.get_piece(start).is_none() {
            return;
        }
        if let Some(taken_piece) = self.pieces.iter().position(|piece| piece.pos == end) {
            self.pieces.remove(taken_piece);
        }
        if let Some(piece) = self.get_piece_mut(start) {
            piece.pos = end;
        }
        self.turn = !self.turn;
    }

    fn pawn_moves(&self, piece: &Piece) -> Vec<Position> {
        let mut out: Vec<Position> = vec![];
        // First square empty
        if self
            .get_piece(piece.pos + piece.direction(Position(0, 1)))
            .is_none()
        {
            out.push(piece.pos + piece.direction(Position(0, 1)));
            // Piece on starting row and second square empty
            if self
                .get_piece(piece.pos + piece.direction(Position(0, 2)))
                .is_none()
                && piece.pos.1
                    == match piece.colour {
                        Colour::White => 1,
                        Colour::Black => 7,
                    }
            {
                out.push(piece.pos + piece.direction(Position(0, 2)))
            }
        }
        if let Some(other_piece) = self.get_piece(piece.pos + piece.direction(Position(1, 1))) {
            if other_piece.colour != piece.colour {
                out.push(piece.pos + piece.direction(Position(1, 1)));
            }
        }
        if let Some(other_piece) = self.get_piece(piece.pos + piece.direction(Position(-1, 1))) {
            if other_piece.colour != piece.colour {
                out.push(piece.pos + piece.direction(Position(-1, 1)));
            }
        }
        if let Some(en_passant) = self.en_passant {
            if en_passant == piece.pos + piece.direction(Position(1, 1))
                || en_passant == piece.pos + piece.direction(Position(-1, 1))
            {
                out.push(en_passant);
            }
        }
        out
    }

    fn knight_moves(&self, piece: &Piece) -> Vec<Position> {
        let mut out: Vec<Position> = vec![];
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
                out.push(piece.pos + direction)
            }
        }
        out
    }

    fn traversal_moves(&self, piece: &Piece) -> Vec<Position> {
        let mut out: Vec<Position> = vec![];
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
                out.push(curr_pos);
                curr_pos += direction;
            }
            if self.check_square_takeable(piece, curr_pos) {
                out.push(curr_pos);
            }
        }
        out
    }

    fn king_moves(&self, piece: &Piece) -> Vec<Position> {
        let mut out: Vec<Position> = vec![];
        let mut test_board = self.clone();
        for i in -1..=1 {
            for j in -1..=1 {
                let pos = Position(i, j);
            }
        }
        todo!()
    }

    fn king_move_safe(&self, piece: &Piece, end: Position, test_board: &mut Board) -> bool {
        let mut out = false;
        if let Some(other_piece) = self.get_piece(end) {
            if other_piece.colour != piece.colour {
                test_board.move_piece_unchecked(piece.pos, end);
                if test_board.check_square_safe(end) {
                    out = true;
                }
                test_board.move_piece_unchecked(end, piece.pos);
            }
        } else {
            test_board.move_piece_unchecked(piece.pos, end);
            if test_board.check_square_safe(end) {
                out = true;
            }
            test_board.move_piece_unchecked(end, piece.pos)
        }
        out
    }

    fn check_square_safe(&self, square: Position) -> bool {
        self.get_all_moves().iter().any(|pos| pos.end == square)
    }

    fn check_square_takeable(&self, piece: &Piece, square: Position) -> bool {
        if let Some(other_piece) = self.get_piece(square) {
            other_piece.colour != piece.colour
        } else {
            true
        }
    }

    fn fmt_board(&self) -> String {
        let templ = "Xx Xx Xx Xx Xx Xx Xx Xx\n";
        let mut outstr = String::from(templ).repeat(8);
        for piece in &self.pieces {
            let index = (7 - piece.pos.1 as usize) * templ.len() + piece.pos.0 as usize * 3;
            outstr.replace_range(
                index..index + 2,
                match piece.kind {
                    PieceKind::Pawn => "P ",
                    PieceKind::Knight => "Kn",
                    PieceKind::Bishop => "B ",
                    PieceKind::Rook => "R ",
                    PieceKind::Queen => "Q ",
                    PieceKind::King => "K ",
                },
            );
        }
        outstr
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.fmt_board())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_white_pawn_moves() {
        let ourpawn = Piece::new(Position(3, 1), Colour::White, PieceKind::Pawn);
        let capturepawn = Piece::new(Position(2, 2), Colour::Black, PieceKind::Pawn);
        let blockingpiece = Piece::new(Position(3, 3), Colour::White, PieceKind::Knight);
        let mut board = Board {
            pieces: vec![ourpawn, capturepawn],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.get_piece_moves(Position(3, 1)).unwrap();
        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&Position(3, 2)));
        assert!(moves.contains(&Position(3, 3)));
        assert!(moves.contains(&Position(2, 2)));
        board.pieces.push(blockingpiece);
        board.get_piece_mut(Position(2, 2)).unwrap().pos = Position(4, 2);
        moves = board.get_piece_moves(Position(3, 1)).unwrap();
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Position(3, 2)));
        assert!(moves.contains(&Position(4, 2)));
        board.get_piece_mut(Position(3, 3)).unwrap().pos = Position(3, 2);
        moves = board.get_piece_moves(Position(3, 1)).unwrap();
        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&Position(4, 2)));
    }

    #[test]
    fn test_black_pawn_moves() {
        let ourpawn = Piece::new(Position(3, 3), Colour::Black, PieceKind::Pawn);
        let capturepawn = Piece::new(Position(2, 2), Colour::White, PieceKind::Pawn);
        let blockingpiece = Piece::new(Position(3, 2), Colour::White, PieceKind::Knight);
        let mut board = Board {
            pieces: vec![ourpawn, capturepawn],
            turn: Colour::Black,
            en_passant: Some(Position(4, 2)),
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.get_piece_moves(Position(3, 3)).unwrap();
        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&Position(3, 2)));
        assert!(moves.contains(&Position(4, 2)));
        assert!(moves.contains(&Position(2, 2)));
        board.pieces.push(blockingpiece);
        moves = board.get_piece_moves(Position(3, 3)).unwrap();
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Position(2, 2)));
        assert!(moves.contains(&Position(4, 2)));
    }

    #[test]
    fn test_knight_moves() {
        let ourknight = Piece::new(Position(3, 1), Colour::Black, PieceKind::Knight);
        let capturepawn = Piece::new(Position(4, 3), Colour::White, PieceKind::Pawn);
        let blockingpiece = Piece::new(Position(1, 2), Colour::Black, PieceKind::Knight);
        let board = Board {
            pieces: vec![ourknight, capturepawn, blockingpiece],
            turn: Colour::Black,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.get_piece_moves(Position(3, 1)).unwrap();
        let mut expectation = vec![
            Position(4, 3),
            Position(2, 3),
            Position(5, 2),
            Position(5, 0),
            Position(1, 0),
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_bishop_moves() {
        let ourbishop = Piece::new(Position(4, 3), Colour::Black, PieceKind::Bishop);
        let captureknight = Piece::new(Position(7, 6), Colour::White, PieceKind::Knight);
        let blockingpiece = Piece::new(Position(6, 1), Colour::Black, PieceKind::King);
        let board = Board {
            pieces: vec![ourbishop, captureknight, blockingpiece],
            turn: Colour::Black,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.get_piece_moves(Position(4, 3)).unwrap();
        let mut expectation = vec![
            Position(5, 4),
            Position(6, 5),
            Position(7, 6),
            Position(3, 2),
            Position(2, 1),
            Position(1, 0),
            Position(5, 2),
            Position(3, 4),
            Position(2, 5),
            Position(1, 6),
            Position(0, 7),
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_rook_moves() {
        let ourrook = Piece::new(Position(4, 3), Colour::White, PieceKind::Rook);
        let captureknight = Piece::new(Position(4, 6), Colour::Black, PieceKind::Knight);
        let blockingpiece = Piece::new(Position(6, 3), Colour::White, PieceKind::King);
        let board = Board {
            pieces: vec![ourrook, captureknight, blockingpiece],
            turn: Colour::White,
            en_passant: None,
            castling_rights: [CastlingRights::new(), CastlingRights::new()],
        };
        let mut moves = board.get_piece_moves(Position(4, 3)).unwrap();
        let mut expectation = vec![
            Position(0, 3),
            Position(1, 3),
            Position(2, 3),
            Position(3, 3),
            Position(5, 3),
            Position(4, 0),
            Position(4, 1),
            Position(4, 2),
            Position(4, 4),
            Position(4, 5),
            Position(4, 6),
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_queen_moves() {
        let ourqueen = Piece::new(Position(4, 3), Colour::White, PieceKind::Queen);
        let captureknight = Piece::new(Position(4, 6), Colour::Black, PieceKind::Knight);
        let blockingpiece = Piece::new(Position(6, 3), Colour::White, PieceKind::King);
        let capturepawn = Piece::new(Position(7, 6), Colour::Black, PieceKind::Pawn);
        let blockingpiece2 = Piece::new(Position(6, 1), Colour::White, PieceKind::Queen);
        let board = Board {
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
        let mut moves = board.get_piece_moves(Position(4, 3)).unwrap();
        let mut expectation = vec![
            Position(0, 3),
            Position(1, 3),
            Position(2, 3),
            Position(3, 3),
            Position(5, 3),
            Position(4, 0),
            Position(4, 1),
            Position(4, 2),
            Position(4, 4),
            Position(4, 5),
            Position(4, 6),
            Position(5, 4),
            Position(6, 5),
            Position(7, 6),
            Position(3, 2),
            Position(2, 1),
            Position(1, 0),
            Position(5, 2),
            Position(3, 4),
            Position(2, 5),
            Position(1, 6),
            Position(0, 7),
        ];
        moves.sort();
        expectation.sort();
        assert_eq!(moves, expectation);
    }

    #[test]
    fn test_king_moves() {
        todo!()
    }
}
