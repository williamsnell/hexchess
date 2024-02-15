use std::sync::{Arc, RwLock};
use rayon::prelude::*;
use hexchesscore::{get_all_valid_moves, Board, Color, Hexagon, Piece};
use bumblebot::random_bot2::{SearchTree, tree_search};
use serde_json;

fn main() {
    let mut board = Board::setup_default_board();
    let mut tree = Arc::new(RwLock::new(SearchTree::new()));
    let _: Vec<()> = (0..100).into_par_iter().map(|x| {
        tree_search(&mut board.clone(), tree.clone());
    }).collect();
    dbg!(serde_json::to_string(&tree));
}    