use crate::board::TransparentBoard;
use crate::error::ChessError;
use crate::piece::{Colour, PieceKind};
use crate::traits::*;
use crate::types::{BoardState, ChessMove};
use itertools::Itertools;
use petgraph::algo::k_shortest_path;
use petgraph::prelude::{DfsPostOrder, Graph, Incoming, NodeIndex, Outgoing};
use petgraph::visit::{EdgeRef, Walker};
use rand::{Rng, rng};

pub struct Engine {
    game: Graph<(TransparentBoard, i64), ChessMove>,
    root: NodeIndex,
}

impl Engine {
    pub fn new(board: &TransparentBoard) -> Self {
        let mut game: Graph<(TransparentBoard, i64), ChessMove> = Graph::new();
        let root = game.add_node((board.clone(), board_value(board)));
        Self { game, root }
    }

    pub fn make_move(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        let mut prune_nodes = vec![];
        for node in self
            .game
            .edges_directed(self.root, Outgoing)
            .filter(|edge| *edge.weight() != chess_move)
            .map(|edge| edge.target())
        {
            prune_nodes.append(&mut self.subtree(node))
        }
        for node in prune_nodes {
            self.game.remove_node(node);
        }
        if let Some(played_move) = self
            .game
            .edges_directed(self.root, Outgoing)
            .find(|edge| *edge.weight() == chess_move)
        {
            self.root = played_move.target();
            Ok(())
        } else {
            let mut new_board = self.game[self.root].0.clone();
            new_board.move_piece(chess_move)?;
            let temp = self.game.add_node((new_board, 0));
            self.game.add_edge(self.root, temp, chess_move);
            self.root = temp;
            Ok(())
        }
    }

    pub fn best_move(&mut self, depth: usize) -> Result<ChessMove, ChessError> {
        for _ in 0..(depth / 2) {
            self.explore_moves();
            self.explore_moves();
            self.update_weights();
            println!("Root weight: {:?}", self.game[self.root].1);
            self.prune_bad_moves();
            //self.prune_illegal_moves();
        }
        for n in self.game.neighbors_directed(self.root, Outgoing) {
            println!("{}", self.game[n].1);
        }
        println!("Min: {}", self.game[self.game.neighbors_directed(self.root, Outgoing).min_by_key(|&n| self.game[n].1).unwrap()].1);
        match self
            .game
            .neighbors_directed(self.root, Outgoing)
            .min_by_key(|&n| self.game[n].1)
        {
            Some(node) => Ok(self.game[self.game.find_edge(self.root, node).unwrap()]),
            None => Err(ChessError::NoMoves),
        }
    }

    const MAXIMUM_EXPLORE_MOVES: usize = 2000;
    const MAXIMUM_DEPTH: usize = 16;
    fn explore_moves(&mut self) {
        let depths = k_shortest_path(&self.game, self.root, None, 1, |_| 1);
        let mut rng = rng();
        let mut nodes: Vec<NodeIndex> = vec![];
        for node in DfsPostOrder::new(&self.game, self.root).iter(&self.game) {
            if self
                .game
                .neighbors_directed(node, Outgoing)
                .next()
                .is_none() && depths.get(&node).unwrap() < &rng.random_range(0..Self::MAXIMUM_DEPTH)
            {
                nodes.push(node);
            }
        }
        let mut nodes_checked = 0;
        println!("Leaf nodes: {}", nodes.len());
        let p = Self::MAXIMUM_EXPLORE_MOVES as f32 / nodes.len() as f32;
        for node in nodes {
            if rng.random::<f32>() < p {
                for &chess_move in self.game[node]
                    .0
                    .all_legal_moves()
                    .iter()
                {
                    let mut new_board = self.game[node].0.clone();
                    new_board.move_piece(chess_move).unwrap();
                    let child = self.game.add_node((new_board, 0));
                    self.game.add_edge(node, child, chess_move);
                }
                nodes_checked += 1;
            }
        }

        println!("Nodes checked: {}", nodes_checked);
    }

    fn update_weights(&mut self) {
        let mut dfs = DfsPostOrder::new(&self.game, self.root);
        while let Some(node) = dfs.next(&self.game) {
            let weights = self
                .game
                .neighbors_directed(node, Outgoing)
                .map(|child| self.game[child].1);
            self.game[node].1 = match self.game[node].0.turn() {
                Colour::White => weights.max().unwrap_or(board_value(&self.game[node].0)),
                Colour::Black => weights.min().unwrap_or(board_value(&self.game[node].0)),
            }
        }
    }

    const PRUNE_THRESHOLD: i64 = 1;
    fn prune_bad_moves(&mut self) {
        let depths = k_shortest_path(&self.game, self.root, None, 1, |_| 1);
        let mut prune_nodes: Vec<NodeIndex> = vec![];
        for (node, parent) in DfsPostOrder::new(&self.game, self.root)
            .iter(&self.game)
            .filter(|n| depths.get(n).unwrap() > &4)
            .map(|n| {
                (
                    n,
                    self.game
                        .neighbors_directed(n, Incoming)
                        .exactly_one()
                        .unwrap(),
                )
            })
        {
            if node == self.root {
                continue;
            }
            match self.game[parent].0.turn() {
                Colour::White => {
                    if self.game[node].1 < self.game[parent].1 - Self::PRUNE_THRESHOLD {
                        prune_nodes.append(&mut self.subtree(node));
                    }
                }
                Colour::Black => {
                    if self.game[node].1 > self.game[parent].1 + Self::PRUNE_THRESHOLD {
                        prune_nodes.append(&mut self.subtree(node));
                    }
                }
            }
        }
        for node in prune_nodes {
            self.game.remove_node(node);
        }
    }

    fn prune_illegal_moves(&mut self) {
        let mut prune_nodes: Vec<NodeIndex> = vec![];
        for node in DfsPostOrder::new(&self.game, self.root)
            .iter(&self.game)
            .map(|n| {
                (
                    n,
                    self.game
                        .neighbors_directed(n, Incoming)
                        .exactly_one()
                        .expect("Node in graph has more than one input: not valid game tree"),
                )
            })
            .filter(|&(n, parent)| {
                !self.game[parent]
                    .0
                    .check_move_legal(self.game[self.game.find_edge(parent, n).unwrap()])
                    .unwrap()
            })
            .map(|(n, _parent)| n)
        {
            prune_nodes.append(&mut self.subtree(node));
        }
        for node in prune_nodes {
            self.game.remove_node(node);
        }
    }

    fn subtree(&self, node: NodeIndex) -> Vec<NodeIndex> {
        DfsPostOrder::new(&self.game, node)
            .iter(&self.game)
            .collect()
    }
}

fn board_value(board: &TransparentBoard) -> i64 {
    let board_state = match (board.get_board_state(), board.turn()) {
        //(BoardState::Check, Colour::Black) => 10,
        //(BoardState::Check, Colour::White) => -10,
        (BoardState::Checkmate, Colour::Black) => return 1000000,
        (BoardState::Checkmate, Colour::White) => return -1000000,
        (BoardState::Stalemate, _) => return 0,
        (_, _) => 0,
    };
    let piece_values: i64 = board
        .get_all_pieces()
        .iter()
        .map(|piece| {
            if piece.colour == Colour::White {
                piece.kind.value() as i64
            } else {
                -(piece.kind.value() as i64)
            }
        })
        .sum();
    let pawn_positions: i64 = board
        .get_all_pieces()
        .iter()
        .filter(|piece| piece.kind == PieceKind::Pawn)
        .map(|piece| {
            (match piece.colour {
                Colour::White => 1,
                Colour::Black => -1,
            } * if (3..=4).contains(&piece.pos.0) && (3..=4).contains(&piece.pos.1) {
                3
            } else if (2..=5).contains(&piece.pos.0) && (2..=5).contains(&piece.pos.1) {
                1
            } else {
                0
            })
        })
        .sum();
    pawn_positions + piece_values * 3 + board_state
}
