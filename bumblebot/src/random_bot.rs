use core::num;
use rand::{thread_rng, Rng, seq::SliceRandom};
use rand_distr::{Distribution, WeightedIndex};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    os::unix::thread,
    sync::{Arc, Mutex, RwLock},
    thread::{Thread, current, sleep}, time::{Instant, Duration}, fs::File, io::Write,
};

const EXPLORATION_PARAMETER: f32 = 0.01;

const SPAWN_THREAD_CUTOFF: u16 = 0;

use hexchesscore::{
    apply_move, check_for_mates, get_all_valid_moves, revert_move, Board, Color, Move,
};

// batched monte-carlo tree-search

// the function is told to spawn n random move trees
// it randomly distributes its available trees between all available moves

// if a move results in a game win/draw/loss:
//      increment the number of games completed counter
//      update the cumulative score tally
//      return the score

// if all games in the batch are completed,
//      increment the counter by the number of trees spawned
//      update the node's average score
//      return the average score

#[derive(Debug)]
pub struct ScoreBoard {
    pub children: HashMap<Move, Arc<RwLock<ScoreBoard>>>,
    pub player_score: u16,
    pub opponent_score: u16,
    pub tally: u16,
}

impl ScoreBoard {
    pub fn new() -> ScoreBoard {
        ScoreBoard {
            children: HashMap::<Move, Arc<RwLock<ScoreBoard>>>::new(),
            player_score: 0,
            opponent_score: 0,
            tally: 1, // initialize to 1 to prevent div by 0 and log(0)
        }
    }
    // handle creating a new scoreboard for this node if one doesn't already exist,
    pub fn retrieve_scoreboard(&mut self, movement: &Move) -> Arc<RwLock<ScoreBoard>> {
        if !self.children.contains_key(movement) {
            self.children
                .insert(*movement, Arc::new(RwLock::new(ScoreBoard::new())));
        }
        Arc::clone(
            self.children
                .get(&movement)
                .expect("Scoreboard should exist, but doesn't"),
        )
    }
    pub fn update_scores(&mut self, num_samples: u16) {
        self.tally += num_samples;
        // opponent becomes player and vice versa, and their score is propagated
        // upwards from the sub simulations
        self.player_score = self
            .children
            .values()
            .fold(0, |acc, e| acc + e.read().unwrap().opponent_score);
        self.opponent_score = self
        .children
        .values()
        .fold(0, |acc, e| acc + e.read().unwrap().player_score);
    }
    pub fn calculate_bias(&self) -> HashMap<Move, f32> {
        self.children
            .iter()
            .map(|(m, x)| {
                let y = x.read().unwrap();
                (m.clone(), y.player_score as f32 / (y.tally as f32)
                    + EXPLORATION_PARAMETER * ((self.tally as f32).log2() / y.tally as f32).sqrt())
            })
            .collect()
    }
    pub fn pick_move(&self) -> Option<Move> {
        let mut highest_bias = f32::MIN;
        let mut best_move = None;
        for (movement, scoreboard) in self.children.iter() {
            let b = scoreboard.read().unwrap();
            let bias = b.tally as f32;
            if bias > highest_bias {
                highest_bias = bias;
                best_move = Some(*movement);
            }
        }
        best_move
    }
}

// tuning parameter for the randomness - bigger divisor gives more smoothness at higher
// computational cost.
const DIVISOR: u16 = 10;
/// See the writeup in docs/sampling/notes.md
/// Will return a vec of equal length to bias -
/// i.e. bias must match num_moves
pub fn get_samples(num_samples: f32, bias: HashMap<Move, f32>) -> HashMap<Move, u16> {
    let b_sum: f32 = bias.values().sum();
    // do a best-attempt at matching the bias distribution with integer number of samples
    let mut choices: HashMap<Move, u16> = bias
        .iter()
        .map(|(m, b)| (m.clone(), (num_samples * b / b_sum) as u16))
        .collect();

    // build a distribution of where our best-attempt is furthest from our desired distribution
    let remainder_bias: HashMap<Move, f32> = bias
        .iter()
        .map(|(m, b)| (m.clone(), num_samples * (b % b_sum)))
        .collect();
    // let debg: Vec<usize> = bias.iter().map(|b| (num_samples * (b % b_sum)).max(1e-4) as usize).collect();
    let mut remainder = (num_samples as u16) - choices.values().sum::<u16>();

    let mut rng = thread_rng();

    // now, divide up the remainder by the bias
    if remainder > 0 {
        if remainder_bias.len() == 1 {
            for (m, choice) in choices.clone() {
                *choices.get_mut(&m).unwrap() += remainder;
            }
        }
        else {
            // I assume at this point that the order of the moves remains constant when I do the 
            // different iterations within this function
            dbg!(remainder_bias.keys());

            let mut choosable_moves = Vec::new();
            let mut biases = Vec::new();

            for (key, val) in remainder_bias {
                choosable_moves.push(key);
                biases.push(val);
            }

            let index =
                WeightedIndex::new(biases).expect("Failed to initialize biased sampler");
    
            while remainder > 0 {
                let allocated = (thread_rng().gen_range(1..remainder + 1) + (DIVISOR - 1)) / DIVISOR;
                dbg!(choices.keys());
                // 
                let chosen_move: Move = choosable_moves[index.sample(&mut rng)];
                *choices.get_mut(&chosen_move).unwrap() += allocated;
    
                remainder -= allocated;
            }
        }
    }                                                                                                                                              

    choices
}

// We want to process all of the remaining moves
// if a given move has a lot of samples, we would like to
// give it its own thread.

// Procedure:
// fn process(list_of_tasks) {
//      for task in tasks
//}

pub fn tree_search(
    board: &mut Board,
    num_searches: u16,
    scoreboard: Arc<RwLock<ScoreBoard>>,
    max_depth: u16,
) {
    let moves = get_all_valid_moves(board);
    let num_moves: u16 = moves.len() as u16;

    let player_color = board.current_player;
    
    // if we haven't yet traversed this nodes' children, our biases
    // will all be 0
    let bias: HashMap<Move, f32>;
    // otherwise, calculate them
    if scoreboard.read().unwrap().children.len() == 0 {
        bias = moves.iter().map(|m| (m.clone(), 1.0)).collect();
    } else {
        bias = scoreboard.read().unwrap().calculate_bias();
    }
    let samples: HashMap<Move, u16> = get_samples(num_searches as f32, bias);

    dbg!(&scoreboard);
    dbg!(&samples);

    let _: Vec<()> = moves
        .par_iter()
        .map(|movement| {
            let sample = samples.get(movement).unwrap();
            let mut board = board.clone();
            let (board, taken_piece) = apply_move(&mut board, *movement);

            let mut sub_scoreboard = scoreboard.write().unwrap().retrieve_scoreboard(movement);

            let _: Vec<()> = (0..*sample).into_par_iter().map(| x | {
                {
                    let mut board = &mut (board.clone());
                    let mut depth = max_depth;
                    
                    while depth > 0 {
                        let moves = get_all_valid_moves(board);
                        if moves.len() == 0 {
                            break;
                        }
                        let rand_move = moves.choose(&mut rand::thread_rng()).unwrap();
                        let (a, b) = apply_move(board, *rand_move);
                        board = a;
                        depth -= 1;
                    };
                    // we have a game ending position
                    let end_type = check_for_mates(& mut board);
    
                    if let Some(end_type) = end_type {
                        let score: u16 = match end_type {
                            hexchesscore::Mate::Checkmate => 4,
                            hexchesscore::Mate::Stalemate => 3, // stalemate is 3/4 of a win
                        };
                        if board.current_player == player_color {
                            sub_scoreboard.write().unwrap().opponent_score += score;
                        } else {
                            sub_scoreboard.write().unwrap().player_score += score;
                        }
                    }
                }
            }).collect();

            sub_scoreboard.write().unwrap().tally += sample;
        })
        .collect();

    // we've got the results back from all of our samples, so lets start
    // updating the scoreboard

    // for some reason this call screws everything up!

    // scoreboard.write().unwrap().update_scores(num_searches);
}


// TODO: keep the scoreboard between moves and continously run search until asked to make a move
pub fn make_a_move(board: &mut Board, timeout_ms: u64) -> Move {
    let end: Instant = Instant::now() + Duration::from_millis(timeout_ms);

    let scoreboard = Arc::new(RwLock::new(ScoreBoard::new()));

    while Instant::now() < end {
        tree_search(board, 1000, scoreboard.clone(), 3000);
        dbg!(&scoreboard.read().unwrap().player_score);
        dbg!(&scoreboard.read().unwrap().tally);
        dbg!(&scoreboard.read().unwrap().pick_move());
    }
    scoreboard.clone().read().unwrap().pick_move().expect("didn't pick a move").clone()
}

fn output_board_representation(board: &Board) {
    let mut f = File::create("../server/debug/board.json").expect("Couldn't open file");

    write!(
        f,
        "{}",
        serde_json::to_string(&board).expect("couldn't serialize board")
    )
    .expect("couldn't write to disk");
}