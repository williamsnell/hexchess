use std::sync::{Arc, RwLock};

use hexchesscore::{
    apply_move, check_for_mates, get_all_valid_moves, revert_move, Board, Color, Move,
};
use rand::{prelude::SliceRandom, thread_rng, Rng};
use rand_distr::{Distribution, WeightedIndex};
use serde::Serialize;

/// basic monte carlo tree search with no (pre-mature) optimizations
///
/// get all available moves
/// if any moves don't have a node on the tree:
///     add a node to the tree
///     complete a random playout (not adding any more nodes)
///     
/// otherwise,
///     calculate the biases for each node
///     randomly pick from the moves based on the computed weights
///     repeat until you get to a move that hasn't been played, at which point follow the previous statement
///
/// once the random playout is complete, propagate the score upwards through all the nodes
///

const MAX_ITERATIONS: u32 = 9999;

#[derive(Debug, Serialize)]
pub struct SearchTree {
    pub wins: i32,
    pub losses: i32,
    pub playouts: i32,
    pub children: Option<Vec<(Move, Arc<RwLock<SearchTree>>)>>,
}

impl SearchTree {
    pub fn new() -> SearchTree {
        SearchTree {
            wins: 0,
            losses: 0,
            playouts: 0,
            children: None,
        }
    }
}

fn evaluate_endgame(board: &mut Board) -> Option<i32> {
    // we have a game ending position
    let end_type = check_for_mates(board);

    if let Some(end_type) = end_type {
        let score: i32 = match end_type {
            hexchesscore::Mate::Checkmate => 4,
            hexchesscore::Mate::Stalemate => 3, // stalemate is 3/4 of a win
        };
        Some(
            score
                * (if board.current_player == Color::White {
                    1
                } else {
                    -1
                }),
        )
    } else {
        None
    }
}
#[derive(Debug)]
pub enum PlayoutErrors {
    MaxIterationsReached,
    ScoreEvaluationError,
    MoveChoiceError,
}
/// Play a game until there is a win, draw, or loss.
/// This function does not expand the tree.
/// Assumes the next move has been pre-applied to the board
fn finish_playout(board: &mut Board) -> Result<i32, PlayoutErrors> {
    // clone the board, since we will be applying lots of moves
    // and don't want to have to revert them all
    let board = &mut board.clone();

    let mut rng = thread_rng();

    for _ in 0..MAX_ITERATIONS {
        let new_moves = get_all_valid_moves(board);

        if new_moves.len() == 0 {
            if let Some(score) = evaluate_endgame(board) {
                return Ok(score);
            } else {
                return Err(PlayoutErrors::ScoreEvaluationError);
            }
        } else {
            // if we haven't reached a terminal state, apply a new move and allow the
            // loop to repeat
            match new_moves.choose(&mut rng) {
                Some(movement) => {
                    let _ = apply_move(board, *movement);
                }
                _ => return Err(PlayoutErrors::MoveChoiceError),
            }
        }
    }

    // we haven't finished the game and have run out of iterations,
    // so we return None to signal a search failure
    Err(PlayoutErrors::MaxIterationsReached)
}

pub fn tally_results(tree: &mut SearchTree, score: i32) {
    match score {
        i32::MIN..=0 => tree.losses -= score,
        1..=i32::MAX => tree.wins += score,
    }
    tree.playouts += 1;
}

fn calculate_bias(children: Vec<&Arc<RwLock<SearchTree>>>) -> Vec<f32> {
    children
        .iter()
        .map(|x| {
            let x = x.read().unwrap();
            (x.wins as f32) / (x.playouts as f32)
                + 1.414 * ((x.playouts as f32).log2() / (x.playouts as f32)).sqrt()
        })
        .collect()
}
/// Choose a move from the possible child moves.
/// Assumes that all child nodes are populated and
/// have all been played at least once.
fn choose_move(tree: &mut SearchTree) -> Option<(Move, Arc<RwLock<SearchTree>>)> {
    // for a placeholder, just pick the first move
    let children: &mut Vec<(Move, Arc<RwLock<SearchTree>>)> = tree.children.as_mut().unwrap();
    let bias = calculate_bias(children.iter().map(|(x, y)| y).collect());

    let mut rng = thread_rng();
    let index = WeightedIndex::new(bias).unwrap();
    let (sampled_move, subtree) = &children[index.sample(&mut rng)];
    Some((sampled_move.clone(), subtree.clone()))
}

pub fn tree_search(board: &mut Board, tree: Arc<RwLock<SearchTree>>) -> Option<i32> {
    // while all the valid moves have nodes, pick one that doesn't
    // have at least 1 playout.
    if tree.read().unwrap().children.is_none() {
        tree.write().unwrap().children = Some(
            get_all_valid_moves(board)
                .into_iter()
                .map(|movement| (movement, Arc::new(RwLock::new(SearchTree::new()))))
                .collect(),
        );
    };
    let mut captured_tree = tree.write().unwrap();
    let children = captured_tree.children.as_mut().unwrap();
    // we hit an endgame position at this point
    if children.len() == 0 {
        let res = evaluate_endgame(board).unwrap();
        tally_results(&mut tree.write().unwrap(), res);
        return Some(res);
    }
    // otherwise, randomize the moves so we don't always do the initial playout in the same
    // order
    children.shuffle(&mut thread_rng());
    for (movement, child_tree) in children {
        let child_tree_read = child_tree.read().unwrap();
        // play all the nodes at this level at least once before
        // starting the random search
        if child_tree_read.playouts == 0 {
            drop(child_tree_read);
            let (board, taken_piece) = apply_move(board, *movement);
            let res = finish_playout(board);
            revert_move(board, *movement, taken_piece);
            if let Ok(score) = res {
                tally_results(&mut child_tree.write().unwrap(), score);
                return Some(score);
            } else {
                // why didn't we get a score if we finished the game?
                dbg!(res);
                return None;
            }
            // since we've completed a playout, we finish the
            // search here
        }
    }
    drop(captured_tree);

    // at this point, we've populated all the nodes at this level,
    // so pick one of the nodes randomly and recurse
    let (random_move, child_tree) = choose_move(&mut tree.write().unwrap()).unwrap();
    let (board, taken_piece) = apply_move(board, random_move);
    let res = tree_search(board, child_tree);
    revert_move(board, random_move, taken_piece);
    return res;
}
