use std::io;

use unchess_lib::{board::TransparentBoard, engine::*, game::GameTree, types::*, *};

fn main() {
    let mut g = GameTree::<TransparentBoard>::starting_board();
    let mut eng = Engine::new(&TransparentBoard::starting_board());
    loop {
        println!("\n{}\n", g.current_board());
        let mut input = String::new();
        match g.turn() {
            piece::Colour::White => {
                io::stdin().read_line(&mut input).unwrap();
                input.pop();
                match AmbiguousMove::try_from(input.as_str()) {
                    Ok(amb_move) => match g.disambiguate_move(amb_move) {
                        Ok(chess_move) => {
                            g.move_piece(chess_move).unwrap();
                            eng.make_move(chess_move).unwrap();

                        }
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                    },
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                }
            }
            piece::Colour::Black => {
                let chess_move = eng.best_move(12).unwrap();
                g.move_piece(chess_move).unwrap();
                eng.make_move(chess_move).unwrap();
            },
        }

        match g.get_board_state() {
            BoardState::Normal => (),
            BoardState::Check => println!("Check!"),
            BoardState::Checkmate => println!("Checkmate!"),
            BoardState::Stalemate => println!("Stalemate!"),
        }
    }
}
