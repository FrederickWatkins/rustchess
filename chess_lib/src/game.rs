use crate::{error::*, traits, types::*};
use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Graph, Incoming, Outgoing};

#[derive(Clone)]
pub struct Game<B: traits::Board> {
    moves: Graph<B, ChessMove>,
    curr: NodeIndex,
}

impl<B: traits::Board> Game<B> {
    pub fn new(board: B) -> Self {
        let mut g = Graph::<B, ChessMove>::new();
        let curr = g.add_node(board);
        Self { moves: g, curr }
    }
}

impl<B: traits::LegalMoveGenerator> traits::LegalMoveGenerator for Game<B> {
    fn check_king_safe(&self, chess_move: ChessMove) -> Result<bool, ChessError> {
        self.moves[self.curr].check_king_safe(chess_move)
    }
}

impl<B: traits::PLegalMoveGenerator + Clone> traits::PLegalMoveGenerator for Game<B> {
    fn all_plegal_moves(&self) -> Vec<ChessMove> {
        self.moves[self.curr].all_plegal_moves()
    }

    fn piece_plegal_moves(&self, pos: Position) -> Result<Vec<ChessMove>, ChessError> {
        self.moves[self.curr].piece_plegal_moves(pos)
    }

    fn check_move_plegal(&self, chess_move: ChessMove) -> Result<bool, ChessError> {
        self.moves[self.curr].check_move_plegal(chess_move)
    }
}

impl<B: traits::Board + Clone> traits::Board for Game<B> {
    fn move_piece(&mut self, chess_move: ChessMove) -> Result<(), ChessError> {
        if let Some(played_move) = self
            .moves
            .edges_directed(self.curr, Outgoing)
            .find(|edge| *edge.weight() == chess_move)
        {
            self.curr = played_move.target();
            Ok(())
        } else {
            let mut new_board = self.moves[self.curr].clone();
            new_board.move_piece(chess_move)?;
            let temp = self.moves.add_node(new_board);
            self.moves.add_edge(self.curr, temp, chess_move);
            self.curr = temp;
            Ok(())
        }
    }

    fn from_fen(fen: &str) -> Result<Self, ChessError> {
        Ok(Self::new(B::from_fen(fen)?))
    }
}

impl<B: traits::Board + traits::LegalMoveGenerator> traits::Game for Game<B> {
    fn undo_move(&mut self) -> Result<(), ChessError> {
        if let Some(prev) = self.moves.edges_directed(self.curr, Incoming).nth(0) {
            self.curr = prev.source();
            Ok(())
        } else {
            Err(ChessError::FirstMove)
        }
    }

    fn from_pgn(pgn: &str) -> Result<Self, ChessError> {
        todo!()
    }
}
