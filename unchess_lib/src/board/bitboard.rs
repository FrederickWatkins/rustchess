//! Bitboard representation of chess board
//!
//! Uses u64s with a bit to represent each square on the chess board, bit 0 representing square a1
//! and bit 64 representing square h8. This is the most performant implementation of a chess board
//! for almost all uses.
use core::fmt;
#[allow(dead_code)]
use std::ops::{Index, IndexMut};
use std::vec::IntoIter;

use super::bit_twiddling;
use crate::enums::{PieceColour, PieceKind};
use crate::error::ChessError;
use crate::notation;
use crate::parser::fen::Fen;
use crate::simple_types::{SimplePiece, SimpleSquare};
use crate::traits::{
    ChessBoard, ChessMove, ChessPiece as _, ChessSquare, LegalMoveGenerator, PLegalMoveGenerator,
};

const PIECE_KINDS: [PieceKind; 6] = [
    PieceKind::King,
    PieceKind::Queen,
    PieceKind::Rook,
    PieceKind::Bishop,
    PieceKind::Knight,
    PieceKind::Pawn,
];

/// Bitwise chess square representation
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitSquare(u64);

impl ChessSquare for BitSquare {
    fn file(&self) -> u8 {
        self.0.trailing_zeros() as u8 % 8
    }

    fn rank(&self) -> u8 {
        self.0.trailing_zeros() as u8 / 8
    }
}

impl From<SimpleSquare> for BitSquare {
    fn from(value: SimpleSquare) -> Self {
        Self::new(value.file(), value.rank())
    }
}

impl BitSquare {
    /// Create new square
    pub const fn new(file: u8, rank: u8) -> Self {
        Self(1 << (file + rank * 8))
    }
}

impl From<BitSquare> for SimpleSquare {
    fn from(value: BitSquare) -> Self {
        SimpleSquare::new(value.file(), value.rank())
    }
}

/// Bitwise encoding of multiple squares
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitSquares(u64);

impl Iterator for BitSquares {
    type Item = BitSquare;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let square = 1 << self.0.trailing_zeros();
        self.0 ^= square;
        Some(BitSquare(square))
    }
}

/// Chess move using [`BitSquare`]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitMove {
    src: BitSquare,
    dest: BitSquare,
    promote_to: Option<PieceKind>,
}

impl ChessMove for BitMove {
    type Square = BitSquare;

    fn src(&self) -> Self::Square {
        self.src
    }

    fn dest(&self) -> Self::Square {
        self.dest
    }

    fn promote_to(&self) -> Option<PieceKind> {
        self.promote_to
    }
}

impl BitMove {
    /// Create new chess move
    pub fn new(src: BitSquare, dest: BitSquare, promote_to: Option<PieceKind>) -> Self {
        Self {
            src,
            dest,
            promote_to,
        }
    }
}

struct BitMoves {
    src: BitSquare,
    dest: BitSquares,
}

impl IntoIterator for BitMoves {
    type Item = BitMove;

    type IntoIter = IntoIter<BitMove>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PieceMap {
    pieces: [BitSquares; 6],
    colour: BitSquares,
}

impl IndexMut<PieceKind> for PieceMap {
    fn index_mut(&mut self, index: PieceKind) -> &mut Self::Output {
        match index {
            PieceKind::King => &mut self.pieces[0],
            PieceKind::Queen => &mut self.pieces[1],
            PieceKind::Rook => &mut self.pieces[2],
            PieceKind::Bishop => &mut self.pieces[3],
            PieceKind::Knight => &mut self.pieces[4],
            PieceKind::Pawn => &mut self.pieces[5],
        }
    }
}

impl Index<PieceKind> for PieceMap {
    type Output = BitSquares;

    fn index(&self, index: PieceKind) -> &Self::Output {
        match index {
            PieceKind::King => &self.pieces[0],
            PieceKind::Queen => &self.pieces[1],
            PieceKind::Rook => &self.pieces[2],
            PieceKind::Bishop => &self.pieces[3],
            PieceKind::Knight => &self.pieces[4],
            PieceKind::Pawn => &self.pieces[5],
        }
    }
}

impl IntoIterator for PieceMap {
    type Item = SimplePiece;

    type IntoIter = IntoIter<SimplePiece>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

impl ChessBoard for PieceMap {
    type Square = BitSquare;
    type Piece = SimplePiece;
    type Move = BitMove;

    fn get_piece(&self, square: BitSquare) -> Result<Self::Piece, ChessError> {
        for kind in PIECE_KINDS {
            if self[kind].0 & square.0 != 0 {
                if self.colour.0 & square.0 != 0 {
                    return Ok(SimplePiece::new(kind, PieceColour::White));
                } else {
                    return Ok(SimplePiece::new(kind, PieceColour::Black));
                }
            }
        }
        Err(ChessError::PieceNotFound(SimpleSquare::from(square)))
    }

    fn all_pieces(&self) -> impl IntoIterator<Item = Self::Piece> {
        *self
    }

    fn move_piece(&mut self, chess_move: Self::Move) -> Result<(), ChessError> {
        let piece = self.get_piece(chess_move.src())?;
        self.remove_piece(chess_move.dest());
        self.remove_piece(chess_move.src());
        if let Some(promote_to) = chess_move.promote_to() {
            self.add_piece(
                chess_move.dest(),
                SimplePiece::new(promote_to, piece.colour()),
            );
        } else {
            self.add_piece(chess_move.dest(), piece);
        }
        Ok(())
    }
}

impl From<Fen> for PieceMap {
    fn from(value: Fen) -> Self {
        let mut board = Self {
            pieces: [BitSquares(0); 6],
            colour: BitSquares(0),
        };
        for (inv_rank_num, rank) in value.layout.iter().enumerate() {
            for (file_num, piece) in rank.iter().enumerate() {
                if let Some(piece) = piece {
                    board.add_piece(
                        BitSquare::new(file_num as u8, 7 - inv_rank_num as u8),
                        *piece,
                    );
                }
            }
        }
        board
    }
}

impl PieceMap {
    fn add_piece(&mut self, square: BitSquare, piece: SimplePiece) {
        self[piece.kind()].0 |= square.0;
        if piece.colour() == PieceColour::White {
            self.colour.0 |= square.0;
        } else {
            self.colour.0 &= !square.0;
        }
    }

    fn remove_piece(&mut self, square: BitSquare) {
        for kind in PIECE_KINDS {
            self[kind].0 &= !square.0;
        }
    }

    fn fmt_board(&self) -> String {
        let mut outstr = String::with_capacity(172);
        for i in (0..8).rev() {
            outstr.push(notation::rank_to_char(i).unwrap());
            for j in 0..8 {
                outstr.push(' ');
                if let Ok(piece) = self.get_piece(BitSquare::new(j, i)) {
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
}

impl fmt::Display for PieceMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.fmt_board())
    }
}

/// Chess bitboard
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitBoard {
    pieces: PieceMap,
    turn: PieceColour,
    en_passant: Option<BitSquare>,
    castling_rights: [bool; 4],
    halfmove_clock: u32,
    fullmove_number: u32,
}

impl ChessBoard for BitBoard {
    type Square = BitSquare;

    type Piece = SimplePiece;

    type Move = BitMove;

    fn get_piece(&self, square: Self::Square) -> Result<Self::Piece, ChessError> {
        self.pieces.get_piece(square)
    }

    fn all_pieces(&self) -> impl IntoIterator<Item = Self::Piece> {
        self.pieces
    }

    fn move_piece(&mut self, chess_move: Self::Move) -> Result<(), ChessError> {
        const KINGSIDE_CASTLE: i8 = 2;
        const QUEENSIDE_CASTLE: i8 = -2;
        const PAWN_DOUBLE_PUSH: i8 = 2;
        self.halfmove_clock += 1;

        match self.get_piece(chess_move.dest()) {
            Ok(_) => self.halfmove_clock = 0,
            Err(ChessError::PieceNotFound(_)) => (),
            Err(e) => return Err(e),
        }

        let piece = self.get_piece(chess_move.src())?;
        self.pieces.move_piece(chess_move)?;

        let rank_offset = chess_move.dest().rank() as i8 - chess_move.src().rank() as i8;
        let file_offset = chess_move.dest().file() as i8 - chess_move.src().file() as i8;

        if piece.kind() == PieceKind::Pawn {
            self.halfmove_clock = 0;
            if rank_offset.abs() == PAWN_DOUBLE_PUSH {
                self.en_passant = Some(BitSquare::new(
                    chess_move.src().file(),
                    (chess_move.src().rank() as i8 + rank_offset / 2) as u8,
                ));
            } else {
                self.en_passant = None;
            }
        }

        if piece.kind() == PieceKind::King {
            match file_offset {
                KINGSIDE_CASTLE => {
                    let src = BitSquare::new(7, chess_move.src().rank());
                    let dest = BitSquare::new(5, chess_move.src().rank());
                    self.pieces.move_piece(BitMove::new(src, dest, None))?;
                },
                QUEENSIDE_CASTLE => {
                    let src = BitSquare::new(0, chess_move.src().rank());
                    let dest = BitSquare::new(3, chess_move.src().rank());
                    self.pieces.move_piece(BitMove::new(src, dest, None))?;
                },
                _ => (),
            }
        }


        self.take_en_passant(piece, chess_move);

        self.update_castling_rights(piece, chess_move);

        self.turn = !self.turn;
        if self.turn == PieceColour::White {
            self.fullmove_number += 1;
        }
        Ok(())
    }
}

// impl PLegalMoveGenerator for BitBoard {
//     fn all_plegal_moves(&self) -> Result<impl IntoIterator<Item = Self::Move>, ChessError> {
//         todo!()
//     }
//
//     fn piece_plegal_moves(&self, square: Self::Square) -> Result<impl IntoIterator<Item = Self::Move>, ChessError> {
//         todo!()
//     }
//
//     fn is_move_plegal(&self, chess_move: Self::Move) -> Result<bool, ChessError> {
//         todo!()
//     }
//
//     fn move_piece_plegal(&mut self, chess_move: Self::Move) -> Result<(), ChessError> {
//         todo!()
//     }
// }

// impl LegalMoveGenerator for BitBoard {
//     fn all_legal_moves(&self) -> Result<impl IntoIterator<Item = Self::Move>, ChessError> {
//         todo!()
//     }
//
//     fn piece_legal_moves(&self, square: Self::Square) -> Result<impl IntoIterator<Item = Self::Move>, ChessError> {
//         todo!()
//     }
//
//     fn is_move_legal(&self, chess_move: Self::Move) -> Result<bool, ChessError> {
//         todo!()
//     }
//
//     fn move_piece_legal(&mut self, chess_move: Self::Move) -> Result<(), ChessError> {
//         todo!()
//     }
//
//     fn state(&self) -> Result<crate::enums::BoardState, ChessError> {
//         todo!()
//     }
//
//     fn disambiguate_move(&self, chess_move: crate::enums::AmbiguousMove) -> Result<Self::Move, ChessError> {
//         todo!()
//     }
// }

impl BitBoard {
    /// Check if move was en passant and if so take other pawn
    fn take_en_passant(&mut self, piece: SimplePiece, chess_move: BitMove) {
        if let Some(taken_pawn_square) = self.en_passant_target(piece, chess_move) {
            self.pieces.remove_piece(taken_pawn_square);
        }
    }

    /// Check if move was en passant and if so return square of pawn to take
    fn en_passant_target(&self, piece: SimplePiece, chess_move: BitMove) -> Option<BitSquare> {
        match self.en_passant {
            Some(en_passant) if piece.kind() == PieceKind::Pawn && chess_move.dest() == en_passant => {
                Some(BitSquare::new(chess_move.dest().file(), chess_move.src().rank()))
            }
            _ => None,
        }
    }

    const KINGSIDE: usize = 0;
    const QUEENSIDE: usize = 1;
    const WHITE_CASTLING_RIGHT_OFFSET: usize = 0;
    const BLACK_CASTLING_RIGHT_OFFSET: usize = 2;
    const fn castling_right_offset(colour: PieceColour) -> usize {
        match colour {
            PieceColour::Black => Self::BLACK_CASTLING_RIGHT_OFFSET,
            PieceColour::White => Self::WHITE_CASTLING_RIGHT_OFFSET,
        }
    }

    fn update_castling_rights(&mut self, piece: SimplePiece, chess_move: BitMove) {
        let castling_offset = Self::castling_right_offset(piece.colour());
        match piece.kind() {
            PieceKind::King => {
                self.castling_rights[castling_offset + Self::KINGSIDE] = false;
                self.castling_rights[castling_offset + Self::QUEENSIDE] = false;
            }
            PieceKind::Rook if chess_move.src().file() == 0 => {
                self.castling_rights[castling_offset + Self::QUEENSIDE] = false;
            }
            PieceKind::Rook if chess_move.src().file() == 7 => {
                self.castling_rights[castling_offset + Self::KINGSIDE] = false;
            }
            _ => (),
        }
    }
}

impl From<Fen> for BitBoard {
    fn from(value: Fen) -> Self {
        let pieces = PieceMap::from(value.clone());
        Self {
            pieces,
            turn: value.turn,
            en_passant: value.en_passant.map(BitSquare::from),
            castling_rights: value.castling_rights,
            halfmove_clock: value.halfmove_clock,
            fullmove_number: value.fullmove_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_starting_board() {
    //     let mut board = PieceMap::starting_board();
    //     println!("{}", board);
    //     board.move_piece(BitMove::new(
    //         BitSquare::new(5, 1),
    //         BitSquare::new(5, 3),
    //         None,
    //     )).unwrap();
    //     println!("{}", board);
    //     panic!();
    // }
}
