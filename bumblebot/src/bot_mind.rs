use std::{
    cmp::{max, min},
    thread::current,
};

use hexchesscore::{get_all_valid_moves, Board, Color, Move, Piece, PieceType};

fn evaluate_board(board: &Board) -> i16 {
    // count each player's pieces
    let mut black_count = 0;
    let mut white_count = 0;
    for (_hex, piece) in &board.occupied_squares {
        let value = match piece.piece_type {
            PieceType::Pawn => 1,
            PieceType::Rook => 4,
            PieceType::Bishop => 3,
            PieceType::Knight => 3,
            PieceType::Queen => 9,
            PieceType::King => 0,
        };
        match piece.color {
            Color::White => white_count += value,
            Color::Black => black_count += value,
        }
    }
    white_count - black_count
}

fn apply_move(board: &mut Board, movement: Move) -> (&mut Board, Option<Piece>) {
    // this function assumes the move is legal. The legality checking
    // should have already happened in the move generation
    let moving_piece = board
        .occupied_squares
        .remove(&movement.start_hex)
        .expect("Piece wasn't present at start hex");
    let taken_piece = board
        .occupied_squares
        .insert(movement.final_hex, moving_piece);
    board.current_player = board.current_player.invert();
    (board, taken_piece)
}

fn revert_move(
    board: &mut Board,
    movement: Move,
    taken_piece: Option<Piece>,
) -> (&mut Board, Option<Piece>) {
    // this function assumes the move is legal. The legality checking
    // should have already happened in the move generation
    let moving_piece = board
        .occupied_squares
        .remove(&movement.final_hex)
        .expect("Piece wasn't present at final hex");
    board
        .occupied_squares
        .insert(movement.start_hex, moving_piece);
    if let Some(taken_piece) = taken_piece {
        board
            .occupied_squares
            .insert(movement.final_hex, taken_piece);
    }
    board.current_player = board.current_player.invert();
    (board, taken_piece)
}

fn evaluate_move(board: &mut Board, movement: Move, color: Color) -> i16 {
    let (new_board, taken_piece) = apply_move(board, movement);
    let rating = evaluate_board(new_board);
    revert_move(new_board, movement, taken_piece);

    rating
}

pub fn minimax(board: &mut Board, movement: Move, depth: i8, mut alpha: i16, mut beta: i16) -> i16 {
    let (new_board, taken_piece) = apply_move(board, movement);
    let mut rating;
    if depth == 0 {
        rating = evaluate_board(board)
    } else {
        // invert now, since we have to do it in either case
        // if the current player is black, then we're white,
        // and want to maximize the value
        let moves = get_all_valid_moves(new_board);
        // if there's a checkmate, it's the highest value possible
        if moves.len() == 0 {
            return i16::MIN;
        }
        if new_board.current_player == Color::Black {
            rating = i16::MIN;
            for valid_move in moves {
                rating = max(
                    rating,
                    minimax(new_board, valid_move, depth - 1, alpha, beta),
                );
                if rating > beta {
                    break;
                }
                alpha = max(alpha, rating);
            }
        } else {
            rating = i16::MAX;
            for valid_move in moves {
                rating = min(
                    rating,
                    minimax(new_board, valid_move, depth - 1, alpha, beta),
                );
                if rating < alpha {
                    break;
                }
                beta = min(beta, rating);
            }
        }
    }
    revert_move(board, movement, taken_piece);
    rating
}

pub fn make_a_move(board: &mut Board, bot_color: Color) -> Move {
    let move_options = get_all_valid_moves(board);
    let mut best_move = move_options[0];
    let mut best_move_rating = evaluate_move(board, best_move, bot_color);

    for player_move in move_options {
        let rating = minimax(board, player_move, 3, i16::MIN, i16::MAX)
            * (if matches!(bot_color, Color::White) {
                1
            } else {
                -1
            });
        dbg!(player_move, rating);
        if rating > best_move_rating {
            best_move_rating = rating;
            best_move = player_move
        }
    }
    best_move
}
