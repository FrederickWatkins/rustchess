use chess_lib::{board::TransparentBoard, types::*, *};

fn main() {
    let mut board = TransparentBoard::starting_board();
    println!("{}", board);
    board
        .move_piece_checked(ChessMove(Position(4, 1), Position(4, 3)))
        .unwrap();
    println!("{}", board);
    board
        .move_piece_checked(ChessMove(Position(3, 6), Position(3, 4)))
        .unwrap();
    println!("{}", board);
    board
        .move_piece_checked(ChessMove(Position(4, 3), Position(3, 4)))
        .unwrap();
    println!("{}", board);
    let mut moves = board.all_legal_moves();
    moves.sort();
    println!("{:?}", moves);
}
