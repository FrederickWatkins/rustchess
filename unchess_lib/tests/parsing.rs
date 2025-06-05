//! Integration tests for parsing
#![allow(clippy::tests_outside_test_module)]
use unchess_lib::notation::pgn_to_moves;

const BYRNE_FISCHER_1956: &str = include_str!("pgn/byrne_fischer_1956.pgn");
const BYRNE_FISCHER_1963: &str = include_str!("pgn/byrne_fischer_1963.pgn");
const FISCHER_BENKO_1963: &str = include_str!("pgn/fischer_benko_1963.pgn");
const FISCHER_MYAGMARSUREN_1967: &str = include_str!("pgn/fischer_myagmarsuren_1967.pgn");
const FISCHER_SPASSKY_1972: &str = include_str!("pgn/fischer_spassky_1972.pgn");

#[test]
fn byrne_fischer_1956() {
    let moves = pgn_to_moves(BYRNE_FISCHER_1956).unwrap();
    assert_eq!(moves.len(), 82);
}

#[test]
fn byrne_fischer_1963() {
    let moves = pgn_to_moves(BYRNE_FISCHER_1963).unwrap();
    assert_eq!(moves.len(), 42);
}

#[test]
fn fischer_benko_1963() {
    let moves = pgn_to_moves(FISCHER_BENKO_1963).unwrap();
    assert_eq!(moves.len(), 41);}

#[test]
fn fischer_myagmarsuren_1967() {
    let moves = pgn_to_moves(FISCHER_MYAGMARSUREN_1967).unwrap();
    assert_eq!(moves.len(), 61);}

#[test]
fn fischer_spassky_1972() {
    let moves = pgn_to_moves(FISCHER_SPASSKY_1972).unwrap();
    assert_eq!(moves.len(), 81);}
