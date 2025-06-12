//! Integration tests for parsing
#![allow(clippy::tests_outside_test_module)]
use unchess_lib::board::piece_list::ChessBoard;
use unchess_lib::notation::pgn_to_moves;
use unchess_lib::traits::{ChessBoard as _, LegalMoveGenerator as _};

const BYRNE_FISCHER_1956: &str = include_str!("pgn/byrne_fischer_1956.pgn");
const BYRNE_FISCHER_1963: &str = include_str!("pgn/byrne_fischer_1963.pgn");
const FISCHER_BENKO_1963: &str = include_str!("pgn/fischer_benko_1963.pgn");
const FISCHER_MYAGMARSUREN_1967: &str = include_str!("pgn/fischer_myagmarsuren_1967.pgn");
const FISCHER_SPASSKY_1972: &str = include_str!("pgn/fischer_spassky_1972.pgn");

fn test_pgn(pgn: &str) {
    let mut board = ChessBoard::starting_board();
    let moves = pgn_to_moves(pgn).unwrap();
    for chess_move in moves {
        let unamb_move = board.disambiguate_move_internal(chess_move).unwrap();
        board.move_piece_legal(unamb_move).unwrap();
    }
}

#[test]
fn byrne_fischer_1956() {
    test_pgn(BYRNE_FISCHER_1956);
}

#[test]
fn byrne_fischer_1963() {
    test_pgn(BYRNE_FISCHER_1963);
}

#[test]
fn fischer_benko_1963() {
    test_pgn(FISCHER_BENKO_1963);
}

#[test]
fn fischer_myagmarsuren_1967() {
    test_pgn(FISCHER_MYAGMARSUREN_1967);
}

#[test]
fn fischer_spassky_1972() {
    test_pgn(FISCHER_SPASSKY_1972);
}
