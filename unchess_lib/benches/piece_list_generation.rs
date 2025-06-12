//! Benchmarks for move generation
#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use unchess_lib::{
    board::piece_list::ChessBoard,
    notation::pgn_to_moves,
    simple_types::SimpleMove,
    traits::{ChessBoard as _, LegalMoveGenerator as _, PLegalMoveGenerator as _},
};

const BYRNE_FISCHER_1956: &str = include_str!("pgn/byrne_fischer_1956.pgn");

fn play_checked_moves(moves: &Vec<SimpleMove>) {
    let mut board = ChessBoard::starting_board();
    for chess_move in moves {
        board.move_piece_legal(black_box(*chess_move)).unwrap();
    }
}

fn play_pchecked_moves(moves: &Vec<SimpleMove>) {
    let mut board = ChessBoard::starting_board();
    for chess_move in moves {
        board.move_piece_plegal(black_box(*chess_move)).unwrap();
    }
}

fn play_unchecked_moves(moves: &Vec<SimpleMove>) {
    let mut board = ChessBoard::starting_board();
    for chess_move in moves {
        board.move_piece(black_box(*chess_move)).unwrap();
    }
}

fn generate_checked_moves(moves: &Vec<SimpleMove>) {
    let mut board = ChessBoard::starting_board();
    for chess_move in moves {
        board.all_legal_moves().unwrap();
        board.move_piece(*chess_move).unwrap();
    }
}

fn generate_pchecked_moves(moves: &Vec<SimpleMove>) {
    let mut board = ChessBoard::starting_board();
    for chess_move in moves {
        board.all_plegal_moves().unwrap();
        board.move_piece(*chess_move).unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut moves: Vec<SimpleMove> = vec![];
    let pgn = pgn_to_moves(BYRNE_FISCHER_1956).unwrap();
    let mut board = ChessBoard::starting_board();
    for ambiguous_move in pgn {
        let chess_move = board.disambiguate_move_internal(ambiguous_move).unwrap();
        moves.push(chess_move);
        board.move_piece(chess_move).unwrap();
    }
    c.bench_function("Legal move checking", |b| b.iter(|| play_checked_moves(&moves)));
    c.bench_function("Pseudo-legal move checking", |b| b.iter(|| play_pchecked_moves(&moves)));
    c.bench_function("Unchecked moving", |b| b.iter(|| play_unchecked_moves(&moves)));
    c.bench_function("Legal move generation", |b| b.iter(|| generate_checked_moves(&moves)));
    c.bench_function("Pseudo-legal move generation", |b| {b.iter(|| generate_pchecked_moves(&moves))});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
