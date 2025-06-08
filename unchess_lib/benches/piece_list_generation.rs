//! Benchmarks for move generation
#![allow(missing_docs)]

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use unchess_lib::{
    board::piece_list::ChessBoard,
    simple_types::SimpleMove,
    traits::{ChessBoard as _, LegalMoveGenerator, PLegalMoveGenerator as _},
};

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
    let mut board = ChessBoard::starting_board();
    for _i in 0..100 {
        let chess_move = board.all_legal_moves().unwrap().into_iter().next().unwrap();
        moves.push(chess_move);
        board.move_piece(chess_move).unwrap();
    }
    c.bench_function("100 checked moves", |b| b.iter(|| play_checked_moves(&moves)));
    c.bench_function("100 psuedo-checked moves", |b| b.iter(|| play_pchecked_moves(&moves)));
    c.bench_function("100 unchecked moves", |b| b.iter(|| play_unchecked_moves(&moves)));
    c.bench_function("100 generated legal moves", |b| {
        b.iter(|| generate_checked_moves(&moves))
    });
    c.bench_function("100 generated pseudo-legal moves", |b| {
        b.iter(|| generate_pchecked_moves(&moves))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
