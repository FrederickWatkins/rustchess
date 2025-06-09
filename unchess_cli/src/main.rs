use clap::{Arg, ArgMatches, Command};
use rustyline::{DefaultEditor, error::ReadlineError};

use unchess_lib::{board::piece_list::ChessBoard, error::ChessError, traits::ChessBoard as _};

fn main() {
    println!("{}", cli().try_get_matches_from(["help"]).unwrap_err());
    let mut repl = Repl::new();
    while !repl.process_input() {}
}

struct Repl {
    board: ChessBoard,
    rl: DefaultEditor,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            board: ChessBoard::starting_board(),
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
            "quit" => return Ok(true),
            _ => unreachable!(),
        }
        Ok(false)
    }

    pub fn new_board(&mut self, fen: Option<&String>) -> Result<(), ChessError> {
        if let Some(fen) = fen {
            self.board = ChessBoard::from_fen(fen)?;
        } else {
            self.board = ChessBoard::starting_board();
        }
        self.show_board();
        Ok(())
    }

    pub fn move_piece(&mut self, chess_move: &str) -> Result<(), ChessError> {
        println!("Not yet implemented");
        Ok(())
    }

    pub fn check_move(&self, chess_move: &str) -> Result<(), ChessError> {
        println!("Not yet implemented");
        Ok(())
    }

    pub fn get_moves(&self, square: &str) -> Result<(), ChessError> {
        println!("Not yet implemented");
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
            Command::new("quit")
                .alias("q")
                .about("Quit the game")
                .help_template(SUBCOMMAND_TEMPLATE),
        )
        .author("Frederick Watkins")
        .version(env!("UNCHESS_FULL_VERSION"))
}
