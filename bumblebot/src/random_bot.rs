
use std::{sync::Mutex, collections::HashMap};
use rand::{Rng, thread_rng};

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

pub fn get_samples(num_moves: usize, num_samples: usize) -> Vec<usize> {
    //      if we're happy sacrificing a few of the extreme edge-cases
    //      (where 1 branch gets more than 2x the mean,) we can do 
    //      the sampling quite easily
    vec![0; num_moves].iter().map(|_| thread_rng().gen_range(0..((num_samples + num_moves/2) / num_moves))).collect()
}




pub fn tree_search(board: &mut Board, search_n_times: usize, scoreboard: ScoreBoard) -> f32 {
    let moves = get_all_valid_moves(board);

    // randomly apportion all the moves
    let num_moves: usize = moves.len();
    let samples: Vec<usize> = vec![0; moves.len()].iter().map(|_| thread_rng().gen_range(0..255)).collect();
    let total: usize = samples.iter().sum();
    // 
    let samples: Vec<usize> = samples.iter().map(|x| x * search_n_times / total).collect();

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