use std::{ops::Add};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Position(i64, i64);

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Colour {
    Black,
    White,
}

impl Colour {
    pub fn direction(self) -> Position {
        if self == Colour::Black {
            Position(0, -1)
        } else {
            Position(0, 1)
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PieceKind {
    Pawn { en_passant: bool },
    Knight,
    Bishop,
    Rook,
    Queen,
    King { castled: bool },
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Piece {
    pos: Position,
    colour: Colour,
    kind: PieceKind,
}

impl Piece {
    fn new(pos: Position, colour: Colour, kind: PieceKind) -> Self {
        Piece { pos, colour, kind }
    }

    fn direction(&self) -> Position {
        self.colour.direction()
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct Game {
    pieces: Vec<Piece>,
}

impl Game {
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

    fn get_piece_moves(&self, pos: Position) -> Result<Vec<Position>, ()> {
        if let Some(piece) = self.get_piece(pos) {
            match piece.kind {
                PieceKind::Pawn { .. } => Ok(self.pawn_moves(piece)),
                PieceKind::Knight => Ok(self.knight_moves(piece)),
                PieceKind::Bishop => Ok(self.bishop_moves(piece)),
                PieceKind::Rook => Ok(self.rook_moves(piece)),
                PieceKind::Queen => Ok(self.queen_moves(piece)),
                PieceKind::King { .. } => Ok(self.king_moves(piece)),
            }
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
        if let Some(piece) = self.get_piece_mut(start) {
            piece.pos = end
        }
    }

    fn pawn_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn knight_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn bishop_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn rook_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn queen_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }

    fn king_moves(&self, piece: &Piece) -> Vec<Position> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let pos1 = Position(3, 5);
        let pos2 = Position(4, 3);
        let result = pos1 + pos2;
        assert_eq!(result, Position(7, 8))
    }
}
