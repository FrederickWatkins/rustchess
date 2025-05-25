use std::hint::black_box;
use chess_lib::{board::TransparentBoard, game, traits::*, types::ChessMove, Board, LegalMoveGenerator};
use criterion::{criterion_group, criterion_main, Criterion};

fn play_checked_moves(moves: &Vec<ChessMove>) {
    let mut game = game::Game::<TransparentBoard>::new(TransparentBoard::starting_board());
    for chess_move in moves {
        game.move_piece_checked(black_box(*chess_move)).unwrap();
    }
}

fn play_pchecked_moves(moves: &Vec<ChessMove>) {
    let mut game = game::Game::<TransparentBoard>::new(TransparentBoard::starting_board());
    for chess_move in moves {
        game.move_piece_pchecked(black_box(*chess_move)).unwrap();
    }
}

fn play_unchecked_moves(moves: &Vec<ChessMove>) {
    let mut game = game::Game::<TransparentBoard>::new(TransparentBoard::starting_board());
    for chess_move in moves {
        game.move_piece(black_box(*chess_move)).unwrap();
    }
}

fn generate_checked_moves(moves: &Vec<ChessMove>) {
    let mut game = game::Game::<TransparentBoard>::new(TransparentBoard::starting_board());
    for chess_move in moves {
        game.all_legal_moves();
        game.move_piece(*chess_move).unwrap();
    }
}

fn generate_pchecked_moves(moves: &Vec<ChessMove>) {
    let mut game = game::Game::<TransparentBoard>::new(TransparentBoard::starting_board());
    for chess_move in moves {
        game.all_plegal_moves();
        game.move_piece(*chess_move).unwrap();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut moves: Vec<ChessMove> = vec![];
    let mut game = game::Game::<TransparentBoard>::new(TransparentBoard::starting_board());
    for _i in 0..100 {
        let chess_move = game.all_legal_moves()[0];
        moves.push(chess_move);
        game.move_piece(chess_move).unwrap();
    }
    c.bench_function("100 checked moves", |b| b.iter(|| play_checked_moves(&moves)));
    c.bench_function("100 psuedo-checked moves", |b| b.iter(|| play_pchecked_moves(&moves)));
    c.bench_function("100 unchecked moves", |b| b.iter(|| play_unchecked_moves(&moves)));
    c.bench_function("100 generated legal moves", |b| b.iter(|| generate_checked_moves(&moves)));
    c.bench_function("100 generated pseudo-legal moves", |b| b.iter(|| generate_pchecked_moves(&moves)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
