use std::{
    cmp::{max, min},
    thread::current, collections::{vec_deque, VecDeque},
};

use hexchesscore::{get_all_valid_moves, Board, Color, Move, Piece, PieceType};

pub fn evaluate_board(board: &Board) -> f32 {
    // count each player's pieces
    let mut black_count = 0.0;
    let mut white_count = 0.0;
    for (_hex, piece) in &board.occupied_squares {
        let value = match piece.piece_type {
            PieceType::Pawn => 1.0,
            PieceType::Rook => 4.0,
            PieceType::Bishop => 3.0,
            PieceType::Knight => 3.0,
            PieceType::Queen => 9.0,
            PieceType::King => 0.0,
        };
        match piece.color {
            Color::White => white_count += value,
            Color::Black => black_count += value,
        }
    }
    dbg!(white_count - black_count);
    (if board.current_player == Color::White {
        1.0
    } else {
        -1.0
    }) * white_count - black_count
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

pub fn negamax(board: &mut Board, depth: i8) -> f32 {
    if depth == 0 {
        evaluate_board(board)
    } else {
        let moves = get_all_valid_moves(board);
        let mut rating = f32::NEG_INFINITY;
        for movement in moves {
            let (new_board, taken_piece) = apply_move(board, movement);
            rating = f32::max(rating, -negamax(new_board, depth - 1));
            revert_move(new_board, movement, taken_piece);
        }
        rating
    }
}

pub fn alpha_beta_prune(board: &mut Board, depth: i8, mut alpha: f32, mut beta: f32) -> f32 {
    dbg!(board.current_player);
    let mut rating;
    if depth == 0 {
        rating = evaluate_board(board)
    } else {
        let moves: VecDeque<Move> = get_all_valid_moves(board).into();

        rating = f32::NEG_INFINITY;

        for valid_move in moves {
            let (new_board, taken_piece) = apply_move(board, valid_move);

            let eval = -alpha_beta_prune(new_board, depth - 1, -beta, -alpha);
            revert_move(board, valid_move, taken_piece);
            if eval > beta {
                break;
            }
            rating = f32::max(alpha, eval);
        }
    }
    rating
}

pub fn alpha_beta_prune_with_best_move(
    board: &mut Board,
    depth: i8,
    mut best_move: Move,
    mut alpha: f32,
    mut beta: f32,
) -> (f32, Move) {
    let mut rating;
    if depth == 0 {
        rating = evaluate_board(board)
    } else {
        let mut moves: VecDeque<Move> = get_all_valid_moves(board).into();

        // reorder the moves to get the assumed best one first
        // TODO check that the move was actually in 
        moves.retain(|x| {x != &best_move});
        moves.push_front(best_move);

        rating = f32::NEG_INFINITY;

        if board.current_player == Color::Black {
            dbg!(board.current_player);
            // start at negative infinity, since if there are no moves available
            // to the opponent, that's the best possible state
            rating = f32::NEG_INFINITY;
            
        } else {
            rating = f32::INFINITY;
            // start at infinity, since if there are no moves available
            // to the opponent, that's the best possible state
            for valid_move in moves {
                let (new_board, taken_piece) = apply_move(board, valid_move);
                let eval = alpha_beta_prune(new_board, depth - 1, alpha, beta);
                if eval < rating {
                    best_move = valid_move;
                    rating = eval;
                }
                revert_move(board, valid_move, taken_piece);
                if rating < alpha {
                    break;
                }
                beta = f32::min(beta, rating);
            }
        }
    }
    (rating, best_move)
}

pub fn iterative_deepening(board: &mut Board, max_depth: i8) -> Move {
    let moves = get_all_valid_moves(board);
    let mut best_move = moves[0];
    
    let mut rating = f32::NAN;
    
    for depth in 0..(max_depth + 1) {
        dbg!(best_move, rating);
        (rating, best_move) = alpha_beta_prune_with_best_move(board, depth, best_move, f32::NEG_INFINITY, f32::INFINITY);
        // evaluate the possible moves via alpha-beta pruning at current depth
        // start with the best move currently identified
    }
    best_move
}

pub fn make_a_move(board: &mut Board, bot_color: Color) -> Move {
    // let move_options = get_all_valid_moves(board);
    // let mut best_move = move_options[0];
    // let mut best_move_rating = evaluate_move(board, best_move, bot_color);

    // for player_move in move_options {
    //     // let rating = alpha_beta_prune(board, player_move, 1, i16::MIN, i16::MAX)
    //     let (new_board, taken_piece) = apply_move(board, player_move);
    //     let rating = iterative_deepening(new_board, 3)
    //         * (if new_board.current_player == Color::White {
    //             -1.0
    //         } else {
    //             1.0
    //         });
    //     revert_move(new_board, player_move, taken_piece);
    //     if rating > best_move_rating {
    //         best_move_rating = rating;
    //         best_move = player_move
    //     }
    // }
    iterative_deepening(board, 2)
}
