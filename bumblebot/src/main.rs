use hexchesscore::{Board, get_all_valid_moves, Move};
use rand::{self, Rng};

fn main() {
    // spool up a bot that will respond to a board state with its 
    // suggested move
    let mut board = Board::setup_default_board();
    dbg!(make_a_move(&mut board));
}

fn make_a_move(board: &mut Board) -> Move {
    let move_options = get_all_valid_moves(board);
    return move_options[rand::thread_rng().gen_range(0..move_options.len())];
}