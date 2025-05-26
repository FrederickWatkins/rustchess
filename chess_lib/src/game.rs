use std::fmt::Display;

use crate::{
    error::*,
    piece::{self, Colour},
    traits::*,
    types::*,
    LegalMoveGenerator,
};
use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Graph, Incoming, Outgoing};
use regex::Regex;

#[derive(Clone)]
pub struct GameTree<B: Board> {
    moves: Graph<B, ChessMove>,
    curr: NodeIndex,
}

impl<B: Board> GameTree<B> {
    pub fn new(board: B) -> Self {
        let mut g = Graph::<B, ChessMove>::new();
        let curr = g.add_node(board);
        Self { moves: g, curr }
    }
}

impl<B: LegalMoveGenerator> LegalMoveGenerator for GameTree<B> {
    fn all_legal_moves(&self) -> Vec<ChessMove> {
        self.moves[self.curr].all_legal_moves()
    }

    fn piece_legal_moves(&self, pos: Position) -> Result<Vec<ChessMove>, ChessError> {
        self.moves[self.curr].piece_legal_moves(pos)
    }

    fn check_move_legal(&self, chess_move: ChessMove) -> Result<bool, ChessError> {
        self.moves[self.curr].check_move_legal(chess_move)
    }

    fn check_king_safe(&self, colour: Colour) -> bool {
        self.moves[self.curr].check_king_safe(colour)
    }
}

impl<B: PLegalMoveGenerator + Clone> PLegalMoveGenerator for GameTree<B> {
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

impl<B: Board + Clone> Board for GameTree<B> {
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

    fn get_piece(&self, pos: Position) -> Option<&piece::Piece> {
        self.moves[self.curr].get_piece(pos)
    }

    fn turn(&self) -> Colour {
        self.moves[self.curr].turn()
    }

    fn starting_board() -> Self {
        Self::new(B::starting_board())
    }
}

impl<B: Board + LegalMoveGenerator> Game<B> for GameTree<B> {
    fn undo_move(&mut self) -> Result<(), ChessError> {
        if let Some(prev) = self.moves.edges_directed(self.curr, Incoming).nth(0) {
            self.curr = prev.source();
            Ok(())
        } else {
            Err(ChessError::FirstMove)
        }
    }

    fn from_pgn(pgn: &str) -> Result<Self, ChessError> {
        let re =
            Regex::new(r"[0-9]\.\s*([a-hNKRQB0-9=x#+O\-]*)\s*([a-hNKRQB0-9=x#+O\-]*)\s*").unwrap();
        let mut g = Self::new(B::starting_board()); // TODO Fix regex to recognise final move
        for s in re.captures_iter(
            &pgn.split(['{', '}'])
                .step_by(2)
                .collect::<Vec<_>>()
                .join(""),
        ) {
            g.move_piece(g.disambiguate_move(AmbiguousMove::try_from(s.extract::<2>().1[0])?)?)?;
            g.move_piece(g.disambiguate_move(AmbiguousMove::try_from(s.extract::<2>().1[1])?)?)?;
        }
        Ok(g)
    }

    fn current_board(&self) -> &B {
        &self.moves[self.curr]
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use crate::{board::TransparentBoard, piece::PieceKind};

    use super::*;

    #[test]
    fn test_pgn() {
        let pgn = r#"1. Nf3 Nf6 2. c4 g6 3. Nc3 Bg7 4. d4 O-O 5. Bf4 d5 6. Qb3 dxc4
        7. Qxc4 c6 8. e4 Nbd7 9. Rd1 Nb6 10. Qc5 Bg4 11. Bg5 {11. Be2
        followed by 12. O-O would have been more prudent. The bishop
        move played allows a sudden crescendo of tactical points to be
        uncovered by Fischer. -- Wade} Na4 {!} 12. Qa3 {On 12. Nxa4
        Nxe4 and White faces considerable difficulties.} Nxc3 {At
        first glance, one might think that this move only helps White
        create a stronger pawn center; however, Fischer's plan is
        quite the opposite. By eliminating the Knight on c3, it
        becomes possible to sacrifice the exchange via Nxe4 and smash
        White's center, while the King remains trapped in the center.}
        13. bxc3 Nxe4 {The natural continuation of Black's plan.}
        14. Bxe7 Qb6 15. Bc4 Nxc3 16. Bc5 Rfe8+ 17. Kf1 Be6 {!! If
        this is the game of the century, then 17...Be6!! must be the
        counter of the century. Fischer offers his queen in exchange
        for a fierce attack with his minor pieces. Declining this
        offer is not so easy: 18. Bxe6 leads to a 'Philidor Mate'
        (smothered mate) with ...Qb5+ 19. Kg1 Ne2+ 20. Kf1 Ng3+
        21. Kg1 Qf1+ 22. Rxf1 Ne2#. Other ways to decline the queen
        also run into trouble: e.g., 18. Qxc3 Qxc5} 18. Bxb6 Bxc4+
        19. Kg1 Ne2+ 20. Kf1 Nxd4+ {This tactical scenario, where a
        king is repeatedly revealed to checks, is sometimes called a
        "windmill."} 21. Kg1 Ne2+ 22. Kf1 Nc3+ 23. Kg1 axb6 24. Qb4
        Ra4 25. Qxb6 Nxd1 26. h3 Rxa2 27. Kh2 Nxf2 28. Re1 Rxe1
        29. Qd8+ Bf8 30. Nxe1 Bd5 31. Nf3 Ne4 32. Qb8 b5 {Every piece
        and pawn of the black camp is defended. The white queen has
        nothing to do.} 33. h4 h5 34. Ne5 Kg7 35. Kg1 Bc5+ 36. Kf1
        Ng3+ {Now Byrne is hopelessly entangled in Fischer's mating
        net.} 37. Ke1 Bb4+ 38. Kd1 Bb3+ 39. Kc1 Ne2+ 40. Kb1 Nc3+
        41. Kc1 Rc2#"#;
        let g = GameTree::<TransparentBoard>::from_pgn(pgn).unwrap();
        assert_eq!(
            g.get_piece(Position::try_from("c1").unwrap()).unwrap().kind,
            PieceKind::King
        );
        assert_eq!(
            g.get_piece(Position::try_from("e5").unwrap()).unwrap().kind,
            PieceKind::Knight
        );
        assert_eq!(
            g.get_piece(Position::try_from("g7").unwrap()).unwrap().kind,
            PieceKind::King
        );
        assert_eq!(
            g.get_piece(Position::try_from("b8").unwrap()).unwrap().kind,
            PieceKind::Queen
        );
        assert_eq!(g.get_board_state(), BoardState::Checkmate)
    }

    #[test]
    fn test_100_move() {
        let mut moves: Vec<ChessMove> = vec![];
        let mut game = GameTree::<TransparentBoard>::new(TransparentBoard::starting_board());
        for _i in 0..100 {
            let chess_move = game.all_legal_moves()[0];
            moves.push(chess_move);
            game.move_piece(chess_move).unwrap();
        }
        let mut game = GameTree::<TransparentBoard>::new(TransparentBoard::starting_board());
        for chess_move in moves {
            game.move_piece_checked(chess_move).unwrap();
        }
    }
}
