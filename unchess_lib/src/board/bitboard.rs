use crate::{
    Board,
    error::ChessError,
    piece::{Colour, PieceKind},
};
use strum::{EnumCount};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct BitChessSquare(u64);

impl TryFrom<(i8, i8)> for BitChessSquare {
    type Error = ChessError;

    fn try_from(value: (i8, i8)) -> Result<Self, Self::Error> {
        if let Some(out) = 1u64.checked_shl((value.0 + value.1 * 8) as u32) {
            Ok(Self(out))
        } else {
            Err(ChessError::InvalidSquare(value.0, value.1))
        }
    }
}

impl From<BitChessSquare> for (i8, i8) {
    fn from(value: BitChessSquare) -> Self {
        ((value.0.trailing_zeros() % 8) as i8, (value.0.trailing_zeros() / 8) as i8)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct BitPiece {
    kind: PieceKind,
    colour: Colour,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct BitBoard {
    piece_boards: [u64; PieceKind::COUNT],
    colour_board: u64,
}

impl BitBoard {
    fn starting_board() -> Self {
        let mut piece_boards = [0u64; PieceKind::COUNT];
        piece_boards[PieceKind::Bishop as usize] = STARTING_BISHOPS;
        piece_boards[PieceKind::Queen as usize] = STARTING_QUEENS;
        piece_boards[PieceKind::King as usize] = STARTING_KINGS;
        piece_boards[PieceKind::Knight as usize] = STARTING_KNIGHTS;
        piece_boards[PieceKind::Pawn as usize] = STARTING_PAWNS;
        piece_boards[PieceKind::Rook as usize] = STARTING_ROOKS;

        Self {
            piece_boards,
            colour_board: STARTING_COLOURS,
        }
    }

    fn get_piece(&self, pos: BitChessSquare) -> Option<BitPiece> {
        Some(BitPiece {
            kind: PieceKind::from_repr(
                self.piece_boards
                    .iter()
                    .enumerate()
                    .find(|&(_i, &piece_board)| piece_board & pos.0 != 0)?
                    .0,
            )
            .unwrap(),
            colour: Colour::from_repr((self.colour_board & pos.0 != 0) as usize).unwrap(),
        })
    }

    fn move_piece(
        &mut self,
        chess_move: crate::types::ChessMove,
    ) -> Result<(), crate::error::ChessError> {
        todo!()
    }

    fn from_fen(fen: &str) -> Result<Self, crate::error::ChessError> {
        todo!()
    }

    fn turn(&self) -> crate::piece::Colour {
        todo!()
    }

    fn get_all_pieces(&self) -> &Vec<crate::piece::Piece> {
        todo!()
    }
}

const fn from_u8s(u: [u8; 8]) -> u64 {
    let mut i = 0;
    let mut out = 0u64;
    while i < u.len() {
        out <<= 8;
        out |= u[i].reverse_bits() as u64;
        i += 1;
    }
    out
}

#[rustfmt::skip]
const STARTING_ROOKS: u64 = from_u8s([
    0b10000001,
    0,
    0,
    0,
    0,
    0,
    0,
    0b10000001,
]);

#[rustfmt::skip]
const STARTING_QUEENS: u64 = from_u8s([
    0b00010000,
    0,
    0,
    0,
    0,
    0,
    0,
    0b00010000,
]);

#[rustfmt::skip]
const STARTING_KNIGHTS: u64 = from_u8s([
    0b01000010,
    0,
    0,
    0,
    0,
    0,
    0,
    0b01000010,
]);

#[rustfmt::skip]
const STARTING_BISHOPS: u64 = from_u8s([
    0b00100100,
    0,
    0,
    0,
    0,
    0,
    0,
    0b00100100,
]);

#[rustfmt::skip]
const STARTING_KINGS: u64 = from_u8s([
    0b00001000,
    0,
    0,
    0,
    0,
    0,
    0,
    0b00001000,
]);

#[rustfmt::skip]
const STARTING_PAWNS: u64 = from_u8s([
    0,
    0b11111111,
    0,
    0,
    0,
    0,
    0b11111111,
    0,
]);

#[rustfmt::skip]
const STARTING_COLOURS: u64 = from_u8s([
    0,
    0,
    0,
    0,
    0,
    0,
    u8::MAX,
    u8::MAX,
]);


const LEFT_WRAP_MASK: [u64; 8] = {
    let mut i = 1;
    let mut out = [u64::MAX; 8];
    while i < 8 {
        let x = u8::MAX;
        out[i] = from_u8s([!(x << (8 - i)); 8]);
        i += 1;
    }
    out
};

const RIGHT_WRAP_MASK: [u64; 8] = {
    let mut i = 1;
    let mut out = [u64::MAX; 8];
    while i < 8 {
        let x = u8::MAX;
        out[i] = from_u8s([!(x >> (8 - i)); 8]);
        i += 1;
    }
    out
};

const fn eval_king_moves(u: u32) -> u64 {
    let mut out = 0u64;
    let king_dirs = [1, 7, 8, 9];
    let king_pos = 1 << u;
    let mut j = 0;
    while j < king_dirs.len() {
        if u >= king_dirs[j] {
            out |= king_pos >> king_dirs[j];
        }
        if u < 64 - king_dirs[j] {
            out |= king_pos << king_dirs[j]
        }
        j += 1;
    }
    if u % 8 == 7 {
        out &= LEFT_WRAP_MASK[1];
    }
    if u % 8 ==0 {
        out &= RIGHT_WRAP_MASK[1];
    }
    out
}

const KING_MOVES: [u64; 64] = {
    let mut i = 0;
    let mut out = [0u64; 64];
    while i < out.len() {
        out[i] = eval_king_moves(i as u32);
        i += 1;
    }
    out
};


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_piece() {
        let board = BitBoard::starting_board();
        assert_eq!(
            board
                .get_piece(BitChessSquare::try_from((0, 0)).unwrap())
                .unwrap(),
            BitPiece {
                kind: PieceKind::Rook,
                colour: Colour::White
            }
        );
        assert_eq!(
            board
                .get_piece(BitChessSquare::try_from((0, 7)).unwrap())
                .unwrap(),
            BitPiece {
                kind: PieceKind::Rook,
                colour: Colour::Black
            }
        );
        assert_eq!(
            board
                .get_piece(BitChessSquare::try_from((3, 1)).unwrap())
                .unwrap(),
            BitPiece {
                kind: PieceKind::Pawn,
                colour: Colour::White
            }
        );
        assert_eq!(
            board
                .get_piece(BitChessSquare::try_from((4, 6)).unwrap())
                .unwrap(),
            BitPiece {
                kind: PieceKind::Pawn,
                colour: Colour::Black
            }
        );
    }
}
