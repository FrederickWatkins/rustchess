use unchess_lib::{board::piece_list::ChessBoard, traits::ChessBoard as _};

fn main() {
    let board = ChessBoard::starting_board();
    println!("{}", board);
}
