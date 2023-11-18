use core::num;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, WeightedIndex};
use std::{collections::HashMap, os::unix::thread, sync::Mutex};

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
    children: HashMap<Move, ScoreBoard>,
    score: f32,
    tally: usize,
}

impl ScoreBoard {
    pub fn new() -> ScoreBoard {
        ScoreBoard {
            children: HashMap::<Move, ScoreBoard>::new(),
            score: 0.0,
            tally: 0,
        }
    }
    // handle creating a new scoreboard for this node if one doesn't already exist,
    pub fn retrieve_scoreboard(&mut self, movement: &Move) -> &mut ScoreBoard {
        if !self.children.contains_key(movement) {
            self.children.insert(*movement, ScoreBoard::new());
        }
        self.children
            .get_mut(&movement)
            .expect("Scoreboard should exist, but doesn't")
    }
    pub fn update_scores(&mut self, num_samples: usize) {
        self.tally += num_samples;
        // this node's score is the sum of all its children's
        // scores
        self.score = self
            .children
            .values()
            .fold(0.0, |acc, e| acc + e.score);
    }
}

// tuning parameter for the randomness - bigger divisor gives more smoothness at higher
// computational cost.
const DIVISOR: usize = 4;
/// See the writeup in docs/sampling/notes.md
/// Will return a vec of equal length to bias -
/// i.e. bias must match num_moves
pub fn get_samples(num_samples: usize, bias: Vec<usize>) -> Vec<usize> {
    let b_sum: usize = bias.iter().sum();
    // do a best-attempt at matching the bias distribution with integer number of samples
    let mut choices: Vec<usize> = bias.iter().map(|b| num_samples * b / b_sum).collect();

    // build a distribution of where our best-attempt is furthest from our desired distribution
    let remainder_bias: Vec<usize> = bias.iter().map(|b| num_samples * b % b_sum).collect();
    let mut remainder = num_samples - choices.iter().sum::<usize>();

    let mut rng = thread_rng();

    // now, divide up the remainder by the bias
    if remainder > 0 {
        let index =
            WeightedIndex::new(remainder_bias).expect("Failed to initialize biased sampler");

        while remainder > 0 {
            let allocated = (thread_rng().gen_range(1..remainder + 1) + (DIVISOR - 1)) / DIVISOR;
            choices[index.sample(&mut rng)] += allocated;

            remainder -= allocated;
        }
    }

    choices
}

pub fn tree_search(
    board: &mut Board,
    num_searches: usize,
    scoreboard: &mut ScoreBoard,
    max_depth: u16,
) {
    let moves = get_all_valid_moves(board);
    let num_moves: usize = moves.len();

    if num_moves == 0 {
        // we have a game ending position
        let end_type = check_for_mates(board);
        dbg!("here");

        if let Some(end_type) = end_type {
            let score: f32 = match end_type {
                hexchesscore::Mate::Checkmate => 1.0,
                hexchesscore::Mate::Stalemate => 3.0 / 4.0,
            };
            let modifier = match board.current_player {
                Color::White => 1.,
                Color::Black => -1.,
            };
            scoreboard.score += score * modifier;
        }
    } else {
        // randomly apportion all the moves

        // use a default bias for now
        let bias = vec![1; num_moves];
        let samples: Vec<usize> = get_samples(num_searches, bias);

        for (movement, sample) in moves.iter().zip(samples) {
            if (sample > 0) & (max_depth > 0) {
                let (board, taken_piece) = apply_move(board, *movement);
                // set up the new scoreboard, or retrieve one that already exists
                let sub_scoreboard = scoreboard.retrieve_scoreboard(movement);
                // todo we need to invert the score each time we propagate it upward
                tree_search(board, sample, sub_scoreboard, max_depth - 1);

                revert_move(board, *movement, taken_piece);
            }
        }
        // we've got the results back from all of our samples, so lets start
        // updating the scoreboard
        scoreboard.update_scores(num_searches);
    }
}
