use std::{collections::HashMap, io, io::Write};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Sign {
    X,
    O,
    None,
}

impl Sign {
    fn complementary(&self) -> Sign {
        match self {
            Sign::X => Sign::O,
            Sign::O => Sign::X,
            Sign::None => Sign::None,
        }
    }
}

#[derive(Debug, Clone)]
enum Player {
    Computer(Sign),
    Human(Sign),
}

#[derive(Clone)]
struct Board {
    positions: HashMap<i8, Option<Sign>>,
}

impl Board {
    fn new() -> Self {
        let mut positions = HashMap::new();
        for i in 1..=9 {
            positions.insert(i, None);
        }
        Board { positions }
    }

    fn display(&self) {
        println!("\x1B[2J\x1B[1;1H");
        for i in 1..=3 {
            for j in 1..=3 {
                let pos = (i - 1) * 3 + j;
                let sign = self.positions.get(&pos).unwrap_or(&None);
                match sign {
                    Some(s) => print!(" {:?} ", s),
                    None => print!(" _ "),
                }
            }
            println!();
        }
    }
}

#[derive(Clone)]
struct Game {
    board: Board,
    turn: Sign,
}

impl Game {
    fn new(player: Player) -> Self {
        Game {
            board: Board::new(),
            turn: match player {
                Player::Computer(sign) | Player::Human(sign) => sign,
            },
        }
    }

    fn check_state(&self) -> bool {
        self.check_winner().is_none() && 
        self.board.positions.values().any(|&pos| pos.is_none())
    }

    fn check_winner(&self) -> Option<Sign> {
        let winning_conditions: Vec<Vec<i8>> = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
            vec![1, 4, 7],
            vec![2, 5, 8],
            vec![3, 6, 9],
            vec![1, 5, 9],
            vec![3, 5, 7],
        ];

        for condition in winning_conditions {
            let first_sign = self.board.positions[&condition[0]];
            if first_sign.is_some()
                && condition
                    .iter()
                    .all(|&pos| self.board.positions[&pos] == first_sign)
            {
                return first_sign;
            }
        }
        None
    }

    fn find_best_move(&self, human: Player) -> i8 {
        let mut best_move = 0;
        let mut best_value = i8::MIN;

        for pos in 1..=9 {
            if self.board.positions[&pos].is_none() {
                let mut new_game = self.clone();
                new_game.board.positions.insert(pos, Some(new_game.turn));

                let move_value = new_game.minimax(
                    new_game.turn.complementary(), 0, human.clone());

                if move_value > best_value {
                    best_value = move_value;
                    best_move = pos;
                }
            }
        }
        best_move
    }

    fn minimax(&self, is_maximizing: Sign, depth: i8, human: Player) -> i8 {
        if let Some(winner) = self.check_winner() {
            let score_adjustment = match winner {
                Sign::X => 10,
                Sign::O => -10,
                _ => 0,
            };
    
            return if matches!(human, Player::Human(Sign::X)) {
                depth - score_adjustment
            } else {
                score_adjustment - depth
            }

        }

        if !self.check_state() {
            return 0;
        }

        let mut best_value = match is_maximizing == self.turn {
            true => i8::MIN,
            false => i8::MAX,
        };

        for pos in 1..=9 {
            if self.board.positions[&pos].is_none() {
                let mut new_game = self.clone();
                new_game.board.positions.insert(pos, Some(is_maximizing));

                let move_value =
                    new_game.minimax(
                        is_maximizing.complementary(), depth + 1, human.clone()
                    );
                    
                best_value = match is_maximizing == self.turn {
                    true => best_value.max(move_value),
                    false => best_value.min(move_value),
                }
            }
        }
        best_value
    }

    fn make_move(&mut self, player: Player, placement: &str) -> Result<(), String> {
        let placement = match placement.parse::<i8>() {
            Ok(s) => s,
            Err(_) => return Err("Invalid Position Try Again (1-9)".to_string()),
        };
        
        if !(placement >= 1 && placement <= 9) {
            return Err("Invalid Position Try Again (1-9)".to_string());
        }

        if self.board.positions[&placement].is_none() {
            self.board
                .positions
                .insert(placement, Some(get_sign(player.clone())));
            Ok(())
        } else {
            Err("Position already occupied".to_string())
        }
    }
}

fn get_sign(player: Player) -> Sign {
    match player {
        Player::Human(sign) | Player::Computer(sign) => sign,
    }
}

fn print(text: &str) {
    print!("{}", text);
    match io::stdout().flush() {
        Ok(_) => {}
        Err(e) => eprintln!("Error flushing stdout: {}", e),
    }
}

fn main() {
    println!("\x1B[2J\x1B[1;1H");
    println!("<----- Tic Tac Toe ----->");

    let player: Player = Player::Human(loop {
        print("Enter Sign (O or X): ");

        let mut sign = String::new();
        if io::stdin().read_line(&mut sign).is_err() {
            eprintln!("Failed to read line.");
            continue;
        }

        let sign = sign.trim().to_uppercase();
        match sign.as_str() {
            "O" => break Sign::O,
            "X" => break Sign::X,
            _ => println!("Invalid sign. Try Again (O or X)\n"),
        }
    });

    let computer: Player = Player::Computer(match player {
        Player::Human(sign) => sign.complementary(),
        Player::Computer(_) => Sign::None,
    });

    let mut game = Game::new(player.clone());
    let mut curr_player = player.clone();

    game.board.display();
    println!("Computer is waitiing for your move");

    while game.check_state() {
        match curr_player {
            Player::Human(_) => {
                let mut input = String::new();
                print(&format!("[{:?}] Enter your move (1-9): ", game.turn));
                io::stdin().read_line(&mut input).expect("Failed to read input");

                if let Err(e) = game.make_move(curr_player.clone(), input.trim()) {
                    game.board.display();
                    println!("{}", e);
                    continue;
                }
            }

            Player::Computer(_) => {
                let best_move = game.find_best_move(player.clone()).to_string();
                game.make_move(computer.clone(), &best_move.clone()).unwrap();
                game.board.display();
                println!("[{:?}] Computer chose {}", game.turn, best_move);
            }
        }

        game.turn = game.turn.complementary();

        if game.check_winner().is_some() {
            game.board.display();
            println!("\n{:?} wins!", curr_player.clone());
            break;
        }

        curr_player = match curr_player {
            Player::Human(_) => computer.clone(),
            Player::Computer(_) => player.clone(),
        }
    }

    if game.check_winner().is_none() {
        game.board.display();
        println!("It's a draw!");
    }
}
