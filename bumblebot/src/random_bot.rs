
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
}

pub fn tree_search(board: &mut Board, search_n_times: u32, scoreboard: ScoreBoard) -> f32 {
    let moves = get_all_valid_moves(board);

    // randomly apportion all the moves
    let num_moves: usize = moves.len();
    let samples = vec![0; moves.len()];
    let samples: Vec<usize> = samples.iter().map(|_| thread_rng().gen_range(0..255)).collect();
    let total: usize = samples.iter().sum();
    dbg!(total);
    // 
    dbg!(&samples);
    let samples: Vec<usize> = samples.iter().map(|x| x * moves.len() * moves.len() / total).collect();
    dbg!(samples);
    1.0
}