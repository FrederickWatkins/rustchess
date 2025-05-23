use crate::chess_move::*;

struct Node {
    chess_move: UnambiguousMove,
    children: Vec<usize>,
}

pub struct MoveTree {
    arena: Vec<Node>,
}