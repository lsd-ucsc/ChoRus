/// Choreographic tik-tak-toe game
extern crate chorus_lib;

use chorus_lib::transport::http::HttpTransportConfig;
use chorus_lib::{
    core::{
        ChoreoOp, Choreography, ChoreographyLocation, Deserialize, Located, LocationSet, Projector,
        Serialize,
    },
    transport::http::HttpTransport,
};

use clap::Parser;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Serialize, Deserialize, Debug)]
enum Status {
    InProgress,
    PlayerWon(char), // 'X' or 'O'
    Tie,
}

impl Status {
    fn is_in_progress(&self) -> bool {
        match self {
            Status::InProgress => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Board {
    board: [char; 9],
}
impl Board {
    fn new() -> Self {
        Self {
            board: core::array::from_fn(|i| std::char::from_digit(i as u32, 10).unwrap()),
        }
    }
    fn draw(&self) {
        fn draw_cell(c: char) {
            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            if c == 'X' {
                stdout
                    .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
                    .unwrap();
            } else if c == 'O' {
                stdout
                    .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
                    .unwrap();
            }
            write!(&mut stdout, "{}", c).unwrap();
            stdout.reset().unwrap();
        }
        for i in (0..3).rev() {
            let offset = i * 3;
            print!("-------------\n| ");
            draw_cell(self.board[offset]);
            print!(" | ");
            draw_cell(self.board[offset + 1]);
            print!(" | ");
            draw_cell(self.board[offset + 2]);
            println!(" |");
        }
        println!("-------------");
    }
    fn check(&self) -> Status {
        // Check rows
        for i in 0..3 {
            let offset = i * 3;
            if self.board[offset] == self.board[offset + 1]
                && self.board[offset + 1] == self.board[offset + 2]
            {
                return Status::PlayerWon(self.board[offset]);
            }
        }
        // Check columns
        for i in 0..3 {
            if self.board[i] == self.board[i + 3] && self.board[i + 3] == self.board[i + 6] {
                return Status::PlayerWon(self.board[i]);
            }
        }
        // Check diagonals
        if self.board[0] == self.board[4] && self.board[4] == self.board[8] {
            return Status::PlayerWon(self.board[0]);
        }
        if self.board[2] == self.board[4] && self.board[4] == self.board[6] {
            return Status::PlayerWon(self.board[2]);
        }
        // Check for tie
        for i in 0..9 {
            if self.board[i] != std::char::from_digit(i as u32, 10).unwrap() {
                continue;
            }
            return Status::InProgress;
        }
        Status::Tie
    }
    fn mark(&mut self, player: char, pos: usize) {
        self.board[pos] = player;
    }
}

#[derive(ChoreographyLocation)]
struct PlayerX;

#[derive(ChoreographyLocation)]
struct PlayerO;

trait Brain {
    fn get_player(&self) -> char;
    fn think(&self, board: &Board) -> Board;
}

struct UserBrain {
    player: char,
}

impl UserBrain {
    fn new(player: char) -> Self {
        Self { player }
    }
}

impl Brain for UserBrain {
    fn get_player(&self) -> char {
        self.player
    }
    fn think(&self, board: &Board) -> Board {
        println!("Current board:");
        board.draw();
        let mut pos = String::new();
        loop {
            println!("Player {}: Enter the number", self.player);
            std::io::stdin().read_line(&mut pos).unwrap();
            if let Ok(pos) = pos.trim().parse::<usize>() {
                if pos >= 9 {
                    println!("{}: Invalid number: {}", self.player, pos);
                }
                if board.board[pos] != std::char::from_digit(pos as u32, 10).unwrap() {
                    println!("{}: Position already taken: {}", self.player, pos);
                }
                // Valid position
                let mut new_board = board.clone();
                new_board.mark(self.player, pos);
                println!("{}: Marked position {}", self.player, pos);
                new_board.draw();
                return new_board;
            }
        }
    }
}

struct MinimaxBrain {
    player: char,
}

impl MinimaxBrain {
    fn new(player: char) -> Self {
        Self { player }
    }
    fn minimax(&self, board: Board, player: char) -> (i32, usize) {
        let status = board.check();
        if status.is_in_progress() {
            let mut best_score = if player == self.player {
                std::i32::MIN
            } else {
                std::i32::MAX
            };
            let mut best_move = 0;
            for i in 0..9 {
                if board.board[i] != std::char::from_digit(i as u32, 10).unwrap() {
                    continue;
                }
                let mut new_board = board.clone();
                new_board.mark(player, i);
                let (score, _) = self.minimax(new_board, if player == 'X' { 'O' } else { 'X' });
                if player == self.player {
                    if score > best_score {
                        best_score = score;
                        best_move = i;
                    }
                } else {
                    if score < best_score {
                        best_score = score;
                        best_move = i;
                    }
                }
            }
            return (best_score, best_move);
        } else {
            match status {
                Status::PlayerWon(player) => {
                    if player == self.player {
                        return (1, 0);
                    } else {
                        return (-1, 0);
                    }
                }
                Status::Tie => return (0, 0),
                _ => unreachable!(),
            }
        }
    }
}

impl Brain for MinimaxBrain {
    fn get_player(&self) -> char {
        self.player
    }
    fn think(&self, board: &Board) -> Board {
        // return the board with the best move
        board.draw();
        println!("Player {}: Thinking...", self.player);
        let (_, best_move) = self.minimax(board.clone(), self.player);
        let mut new_board = board.clone();
        new_board.mark(self.player, best_move);
        println!("{}: Marked position {}", self.player, best_move);
        new_board.draw();
        return new_board;
    }
}

struct TicTacToeChoreography {
    brain_for_x: Located<Box<dyn Brain>, PlayerX>,
    brain_for_o: Located<Box<dyn Brain>, PlayerO>,
}

impl Choreography for TicTacToeChoreography {
    type L = LocationSet!(PlayerX, PlayerO);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> () {
        let mut board = Board::new();
        loop {
            let board_x = op.locally(PlayerX, |un| {
                let brain = un.unwrap(&self.brain_for_x);
                return brain.think(&board);
            });
            board = op.broadcast(PlayerX, board_x);
            if !board.check().is_in_progress() {
                break;
            }
            let board_o = op.locally(PlayerO, |un| {
                let brain = un.unwrap(&self.brain_for_o);
                return brain.think(&board);
            });
            board = op.broadcast(PlayerO, board_o);
            if !board.check().is_in_progress() {
                break;
            }
        }
        let status = board.check();
        match status {
            Status::PlayerWon('X') => {
                op.locally(PlayerX, |_| println!("You win!"));
                op.locally(PlayerO, |_| println!("You lose"));
            }
            Status::PlayerWon('O') => {
                op.locally(PlayerX, |_| println!("You lose"));
                op.locally(PlayerO, |_| println!("You win!"));
            }
            Status::Tie => println!("Tie!"),
            _ => unreachable!(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Player to play as (X or O)
    player: char,
    /// Hostname to listen on
    hostname: String,
    /// Port to listen on
    port: u16,
    /// Hostname of opponent
    opponent_hostname: String,
    /// Port of opponent
    opponent_port: u16,
    /// Use minimax brain instead of user brain
    #[arg(short, long)]
    minimax_brain: bool,
}

fn main() {
    let args = Args::parse();
    let brain: Box<dyn Brain> = if args.minimax_brain {
        Box::new(MinimaxBrain::new(args.player))
    } else {
        Box::new(UserBrain::new(args.player))
    };

    match args.player {
        'X' => {
            let config = HttpTransportConfig::for_target(
                PlayerX,
                (args.hostname.as_str().to_string(), args.port),
            )
            .with(
                PlayerO,
                (
                    args.opponent_hostname.as_str().to_string(),
                    args.opponent_port,
                ),
            );

            let transport = HttpTransport::new(config);
            let projector = Projector::new(PlayerX, transport);
            projector.epp_and_run(TicTacToeChoreography {
                brain_for_x: projector.local(brain),
                brain_for_o: projector.remote(PlayerO),
            });
        }
        'O' => {
            let config = HttpTransportConfig::for_target(
                PlayerO,
                (args.hostname.as_str().to_string(), args.port),
            )
            .with(
                PlayerX,
                (
                    args.opponent_hostname.as_str().to_string(),
                    args.opponent_port,
                ),
            );

            let transport = HttpTransport::new(config);
            let projector = Projector::new(PlayerO, transport);
            projector.epp_and_run(TicTacToeChoreography {
                brain_for_x: projector.remote(PlayerX),
                brain_for_o: projector.local(brain),
            });
        }
        _ => {
            println!("Invalid player; must be X or O");
            std::process::exit(1);
        }
    }
}
