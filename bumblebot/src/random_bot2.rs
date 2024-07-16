use hexchesscore::{
    apply_move, check_for_mates, get_all_valid_moves, revert_move, Board, Color, Move,
};
use rand::{prelude::SliceRandom, thread_rng, Rng};
use rand_distr::{Distribution, WeightedIndex};

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

const MAX_ITERATIONS: u32 = 999;

#[derive(Debug)]
pub struct SearchTree {
    pub wins: i32,
    pub losses: i32,
    pub playouts: i32,
    pub children: Option<Vec<(Move, SearchTree)>>,
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
/// Play a game until there is a win, draw, or loss.
/// This function does not expand the tree.
/// Assumes the next move has been pre-applied to the board
fn finish_playout(board: &mut Board) -> Option<i32> {
    // clone the board, since we will be applying lots of moves
    // and don't want to have to revert them all
    let board = &mut board.clone();

    let mut rng = thread_rng();

    for _ in 0..MAX_ITERATIONS {
        let new_moves = get_all_valid_moves(board);

        if new_moves.len() == 0 {
            return evaluate_endgame(board);
        } else {
            // if we haven't reached a terminal state, apply a new move and allow the
            // loop to repeat
            let movement = new_moves.choose(&mut rng).unwrap();
            // assume that we're modifying the board in-place; this needs to be verified
            let _ = apply_move(board, *movement);
        }
    }

    // we haven't finished the game and have run out of iterations,
    // so we return None to signal a search failure
    None
}

pub fn tally_results(tree: &mut SearchTree, score: i32) {
    match score {
        i32::MIN..=0 => tree.losses -= score,
        1..=i32::MAX => tree.wins += score,
    }
    tree.playouts += 1;
}

fn calculate_bias(children: Vec<&SearchTree>) -> Vec<f32> {
    children
        .iter()
        .map(|x| {
            (x.wins as f32) / (x.playouts as f32)
                + 1.414 * ((x.playouts as f32).log2() / (x.playouts as f32)).sqrt()
        })
        .collect()
}
/// Choose a move from the possible child moves.
/// Assumes that all child nodes are populated and
/// have all been played at least once.
fn choose_move(tree: &mut SearchTree) -> Option<&mut (Move, SearchTree)> {
    // for a placeholder, just pick the first move
    let children: &mut Vec<(Move, SearchTree)> = tree.children.as_mut().unwrap();
    let bias = calculate_bias(children.iter().map(|(_x, y)| y).collect());

    let mut rng = thread_rng();
    let index = WeightedIndex::new(bias).unwrap();

    Some(&mut children[index.sample(&mut rng)])
}

pub fn tree_search(board: &mut Board, tree: &mut SearchTree) -> Option<i32> {
    // while all the valid moves have nodes, pick one that doesn't
    // have at least 1 playout.

    if let Some(children) = &mut tree.children {
        if children.len() == 0 {
            let res = evaluate_endgame(board).unwrap();
            tally_results(tree, res);
            return Some(res);
        }
        children.shuffle(&mut thread_rng());
        for (movement, child_tree) in children {
            // play all the nodes at this level at least once before
            // starting the random search
            if child_tree.playouts == 0 {
                let (board, taken_piece) = apply_move(board, *movement);
                let res = finish_playout(board);
                revert_move(board, *movement, taken_piece);
                if let Some(score) = res {
                    tally_results(child_tree, score);
                }
                // since we've completed a playout, we finish the
                // search here
                return res;
            }
        }
        // at this point, we've populated all the nodes at this level,
        // so pick one of the nodes randomly and recurse
        let (random_move, child_tree) = choose_move(tree).unwrap();
        let (board, taken_piece) = apply_move(board, *random_move);
        let res = tree_search(board, child_tree);
        revert_move(board, *random_move, taken_piece);
        return res;
    }
    // otherwise, populate this layer, and then call the function again
    else {
        tree.children = Some(
            get_all_valid_moves(board)
                .into_iter()
                .map(|movement| (movement, SearchTree::new()))
                .collect(),
        );

        return tree_search(board, tree);
    }

    // if there are no valid moves, do the endgame scoring

    // otherwise, when we hit an unsearched node, do a full playout
    //
}
