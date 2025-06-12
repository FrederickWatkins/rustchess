//! Bitboard representation of chess board
//!
//! Uses u64s with a bit to represent each square on the chess board, bit 0 representing square a1
//! and bit 64 representing square h8. This is the most performant implementation of a chess board
//! for almost all uses.
