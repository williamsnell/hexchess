use hexchesscore::{self, Hexagon};
use serde_json;
use std::fs::File;
use std::io::Write;

fn main() {
    let rook_moves: Vec<Hexagon> =
        hexchesscore::moves::RookMoves::new(Hexagon::new("L5").unwrap()).collect();
    let moves_json = serde_json::to_string(&rook_moves).unwrap();
    let json = format!("{{\"moves\": {moves_json}}}");
    let mut f = File::create("../../server/moves.json").unwrap();
    write!(f, "{}", json).unwrap();
}
