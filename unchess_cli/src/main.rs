use clap::{Arg, ArgMatches, Command};
use colored::*;
use rustyline::{DefaultEditor, error::ReadlineError};

use unchess_lib::{
    board::piece_list::PieceListBoard,
    error::ChessError,
    notation,
    simple_types::SimpleSquare,
    traits::{ChessBoard as _, ChessMove, ChessPiece as _, ChessSquare as _, LegalMoveGenerator},
};

fn main() {
    println!("{}", cli().try_get_matches_from(["help"]).unwrap_err());
    let mut repl = Repl::new();
    while !repl.process_input() {}
}

struct Repl {
    board: PieceListBoard,
    rl: DefaultEditor,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            board: PieceListBoard::starting_board(),
            rl: DefaultEditor::new().expect("Failed to initialize repl"),
        }
    }

    pub fn process_input(&mut self) -> bool {
        match self.rl.readline(">") {
            Ok(buffer) => {
                self.rl.add_history_entry(&buffer).unwrap();
                let args = shlex::split(&buffer).unwrap();
                match cli().try_get_matches_from(args) {
                    Ok(matches) => match self.process_command(&matches) {
                        Ok(true) => return true,
                        Err(e) => println!("{}", e),
                        _ => (),
                    },
                    Err(e) => println!("{}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                return true;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                return true;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                return true;
            }
        }
        false
    }

    pub fn process_command(&mut self, args: &ArgMatches) -> Result<bool, ChessError> {
        let command = args.subcommand().unwrap();
        match command.0 {
            "new" => self.new_board(command.1.get_one::<String>("fen"))?,
            "move" => self.move_piece(command.1.get_one::<String>("PGN").unwrap())?,
            "check" => self.check_move(command.1.get_one::<String>("PGN").unwrap())?,
            "get" => self.get_moves(command.1.get_one::<String>("SQUARE").unwrap())?,
            "show" => self.show_board(),
            "print-fen" => println!("{}", self.board.as_fen_str()?),
            "quit" => return Ok(true),
            _ => unreachable!(),
        }
        Ok(false)
    }

    pub fn new_board(&mut self, fen: Option<&String>) -> Result<(), ChessError> {
        if let Some(fen) = fen {
            self.board = PieceListBoard::from_fen(fen)?;
        } else {
            self.board = PieceListBoard::starting_board();
        }
        self.show_board();
        Ok(())
    }

    pub fn move_piece(&mut self, chess_move: &str) -> Result<(), ChessError> {
        self.board.move_piece(self.board.disambiguate_move_pgn(chess_move)?)?;
        self.show_board();
        self.board_state()?;
        Ok(())
    }

    pub fn check_move(&self, chess_move: &str) -> Result<(), ChessError> {
        if self
            .board
            .is_move_legal(self.board.disambiguate_move_pgn(chess_move)?)?
        {
            println!("Move {} legal", chess_move);
        } else {
            println!("Illegal move {chess_move}");
        }
        Ok(())
    }

    pub fn get_moves(&self, square: &str) -> Result<(), ChessError> {
        let square = SimpleSquare::from_pgn_str(square)?;
        let dest_squares: Vec<SimpleSquare> = self
            .board
            .piece_legal_moves(square)?
            .into_iter()
            .map(|chess_move| chess_move.dest())
            .collect();
        for i in (0..8).rev() {
            print!("{}", notation::rank_to_char(i).unwrap());
            for j in 0..8 {
                print!(" ");
                self.print_square(&dest_squares, square, SimpleSquare::new(j, i))?;
            }
            println!();
        }

        print!("  ");
        for j in 0..8 {
            print!("{}", notation::file_to_char(j).unwrap());
            print!(" ");
        }
        println!();
        Ok(())
    }

    fn print_square(
        &self,
        dest_squares: &[SimpleSquare],
        src_square: SimpleSquare,
        curr_square: SimpleSquare,
    ) -> Result<(), ChessError> {
        match self.board.get_piece(curr_square) {
            Ok(piece) if dest_squares.contains(&piece.square()) => {
                print!("{}", piece.as_fen().to_string().on_blue().red())
            }
            Ok(piece) if piece.square() == src_square => print!("{}", piece.as_fen().to_string().magenta().bold()),
            Ok(piece) => print!("{}", piece),
            Err(ChessError::PieceNotFound(_)) => {
                let mut s = ' ';
                if (curr_square.rank() + curr_square.file()) % 2 == 1 {
                    s = '◼';
                }
                if dest_squares.contains(&curr_square) {
                    print!("{}", s.to_string().on_blue());
                } else {
                    print!("{}", s);
                }
            }
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn board_state(&self) -> Result<(), ChessError> {
        match self.board.state()? {
            unchess_lib::enums::BoardState::Normal => (),
            unchess_lib::enums::BoardState::Check => println!("{}", "Check!".magenta().bold()),
            unchess_lib::enums::BoardState::Stalemate => println!("{}", "Stalemate!".bold()),
            unchess_lib::enums::BoardState::Checkmate => println!("{}", "Checkmate!".red().bold()),
        }
        Ok(())
    }

    pub fn show_board(&self) {
        println!("{}", self.board);
    }
}

fn cli() -> Command {
    // strip out usage
    const PARSER_TEMPLATE: &str = "\
        unchess-cli v{version}\n\
        {author}\n\n\
        {all-args}
    ";
    // strip out name/version
    const SUBCOMMAND_TEMPLATE: &str = "\
        {about-with-newline}\n\
        {usage-heading}\n    {usage}\n\
        \n\
        {all-args}{after-help}\
    ";

    Command::new("unchess-cli")
        .bin_name("chess-cli")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_value_name("Commands")
        .subcommand_help_heading("Commands")
        .help_template(PARSER_TEMPLATE)
        .subcommand(
            Command::new("new")
                .about("Create a new board")
                .help_template(SUBCOMMAND_TEMPLATE)
                .arg(
                    Arg::new("fen")
                        .short('f')
                        .value_name("FEN")
                        .help("Optional FEN to start the board from."),
                ),
        )
        .subcommand(
            Command::new("move")
                .about("Move a piece")
                .help_template(SUBCOMMAND_TEMPLATE)
                .arg(Arg::new("PGN").help(
                    "The standard PGN for the move. Overspecification will be handled but ambiguous moves will error.",
                ))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("get")
                .about("Get legal moves for a piece")
                .help_template(SUBCOMMAND_TEMPLATE)
                .arg(Arg::new("SQUARE").help("The square of the piece to get moves for."))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("check")
                .about("Check if a move is legal")
                .help_template(SUBCOMMAND_TEMPLATE)
                .arg(Arg::new("PGN").help(
                    "The standard PGN for the move. Overspecification will be handled but ambiguous moves will error.",
                ))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("show")
                .about("Show the current board state")
                .help_template(SUBCOMMAND_TEMPLATE),
        )
        .subcommand(
            Command::new("print-fen")
                .about("Print the current board state in fen format")
                .help_template(SUBCOMMAND_TEMPLATE),
        )
        .subcommand(
            Command::new("quit")
                .alias("q")
                .about("Quit the game")
                .help_template(SUBCOMMAND_TEMPLATE),
        )
        .author("Frederick Watkins")
        .version(env!("UNCHESS_FULL_VERSION"))
}
