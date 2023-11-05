
use std::{sync::Mutex, collections::HashMap, os::unix::thread};
use rand::{Rng, thread_rng};
use rand_distr::{WeightedIndex, Distribution};

use hexchesscore::{apply_move, revert_move, Move, Board, get_all_valid_moves};

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

pub struct ScoreBoard {
    children: HashMap<Move, ScoreBoard>,
    score: f32,
    tally: i32
}

impl ScoreBoard {
    pub fn new() -> ScoreBoard {
        ScoreBoard { children: HashMap::<Move, ScoreBoard>::new(), score: 0.0, tally: 0 }
    }
    // handle creating a new scoreboard for this node if one doesn't already exist,
    //  
}

// tuning parameter for the randomness - bigger divisor gives more smoothness at higher
// computational cost.
const DIVISOR: usize = 4;
/// See the writeup in docs/sampling/notes.md
/// Will return a vec of equal length to bias
pub fn get_samples(num_moves: usize, num_samples: usize, bias: Vec<usize>) -> Vec<usize> {
    let b_sum: usize = bias.iter().sum();
    // do a best-attempt at matching the bias distribution with integer number of samples
    let mut choices: Vec<usize> = bias.iter().map(|b| num_samples * b / b_sum).collect();

    // build a distribution of where our best-attempt is furthest from our desired distribution
    let remainder_bias: Vec<usize> = bias.iter().map(|b| num_samples * b % b_sum).collect();
    let mut remainder = num_samples - choices.iter().sum::<usize>();

    let mut rng = thread_rng();

    // now, divide up the remainder by the bias
    let index = WeightedIndex::new(remainder_bias)
        .expect("Failed to initialize biased sampler");
    
    while remainder > 0 {
        let allocated = (thread_rng().gen_range(1..remainder+1) + (DIVISOR - 1)) / DIVISOR;
        choices[index.sample(&mut rng)] += allocated;

        remainder -= allocated;
    }

    choices
}




pub fn tree_search(board: &mut Board, num_searches: usize, scoreboard: ScoreBoard) -> f32 {
    let moves = get_all_valid_moves(board);

    // randomly apportion all the moves
    let num_moves: usize = moves.len();
    let samples: Vec<usize> = vec![0; moves.len()].iter().map(|_| thread_rng().gen_range(0..255)).collect();
    let total: usize = samples.iter().sum();
    // 
    let samples: Vec<usize> = get_samples(num_moves, num_searches, vec![1; num_moves]);


    dbg!(&samples);
    let actual_total: usize = samples.iter().sum();
    dbg!(actual_total);

    for (i, movement) in moves.iter().enumerate() {
        let (board, taken_piece) = apply_move(board, *movement);
        // set up the new scoreboard, or retrieve one that already exists
        // tree_search(board, samples[i], scoreboard);

        revert_move(board, *movement, taken_piece);
    }
    1.0
}