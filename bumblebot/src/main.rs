use bumblebot::random_bot2::{tree_search, SearchTree};
use hexchesscore::{get_all_valid_moves, Board, Color, Hexagon, Piece};
use rayon::prelude::*;
use serde_json;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, RwLock};

fn main() -> std::io::Result<()> {
    let mut board = Board::setup_default_board();
    let mut tree = Arc::new(RwLock::new(SearchTree::new()));
    loop {
        let _: Vec<()> = (0..50)
            .into_par_iter()
            .map(|x| {
                dbg!(x);
                tree_search(&mut board.clone(), tree.clone());
            })
            .collect();
        let file = File::create("out.json")?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &tree);
        writer.flush()?;
    }
    Ok(())
}
