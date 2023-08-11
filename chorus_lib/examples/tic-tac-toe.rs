/// Choreographic tik-tak-toe game
/// Based on https://medium.com/aimonks/rust-tic-tac-toe-game-with-minimax-algorithm-dc64745974b6
extern crate chorus_lib;

use chorus_lib::{
    core::{Choreography, ChoreographyLocation, Deserialize, Located, Projector, Serialize},
    transport::http::HttpTransport,
};
use clap::Parser;
use std::{collections::HashMap, io::Write, rc::Rc};
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
    fn think(&self, board: Board) -> Board;
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
    fn think(&self, board: Board) -> Board {
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

struct TicTacToeChoreography {
    brain_for_x: Located<Rc<dyn Brain>, PlayerX>,
    brain_for_y: Located<Rc<dyn Brain>, PlayerO>,
}

impl Choreography for TicTacToeChoreography {
    fn run(self, op: &impl chorus_lib::core::ChoreoOp) -> () {
        let mut board = Board::new();
        loop {
            let board_x = op.locally(PlayerX, |un| {
                let brain = un.unwrap(self.brain_for_x.clone());
                return brain.think(board);
            });
            board = op.broadcast(PlayerX, board_x);
            if !board.check().is_in_progress() {
                break;
            }
            let board_o = op.locally(PlayerO, |un| {
                let brain = un.unwrap(self.brain_for_y.clone());
                return brain.think(board);
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
    player: char,
    hostname: String,
    port: u16,
    opponent_hostname: String,
    opponent_port: u16,
}

fn main() {
    let args = Args::parse();
    match args.player {
        'X' => {
            let mut config = HashMap::new();
            config.insert(PlayerX.name(), (args.hostname.as_str(), args.port));
            config.insert(
                PlayerO.name(),
                (args.opponent_hostname.as_str(), args.opponent_port),
            );
            let transport = HttpTransport::new(PlayerX.name(), &config);
            let projector = Projector::new(PlayerX, transport);
            projector.epp_and_run(TicTacToeChoreography {
                brain_for_x: projector.local(Rc::new(UserBrain::new('X'))),
                brain_for_y: projector.remote(PlayerO),
            });
        }
        'O' => {
            let mut config = HashMap::new();
            config.insert(PlayerO.name(), (args.hostname.as_str(), args.port));
            config.insert(
                PlayerX.name(),
                (args.opponent_hostname.as_str(), args.opponent_port),
            );
            let transport = HttpTransport::new(PlayerO.name(), &config);
            let projector = Projector::new(PlayerO, transport);
            projector.epp_and_run(TicTacToeChoreography {
                brain_for_x: projector.remote(PlayerX),
                brain_for_y: projector.local(Rc::new(UserBrain::new('O'))),
            });
        }
        _ => unreachable!(),
    }
}