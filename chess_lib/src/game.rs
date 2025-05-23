use crate::board;
use crate::move_tree;

pub struct Game {
    board: board::Board,
    moves: move_tree::MoveTree,
}