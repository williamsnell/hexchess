use api::OutgoingMessage;
use std::{
    collections::{VecDeque},
    time::Duration,
};
use tokio::{self, sync::mpsc};
use std::time::Instant;
use warp::ws::Message;

use hexchesscore::{get_all_valid_moves, Board, Color, Move, PieceType, apply_move, revert_move};

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
    (if board.current_player == Color::White {
        1.0
    } else {
        -1.0
    }) * (white_count - black_count)
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

pub fn send_board(transmitter: &mpsc::UnboundedSender<Message>, board: Board) {
    let _result = transmitter.send(Message::text(
        serde_json::to_string(&OutgoingMessage::BoardState { board: board }).unwrap(),
    ));
}

pub fn alpha_beta_prune(
    board: &mut Board,
    depth: i8,
    mut alpha: f32,
    beta: f32,
    timeout: Instant
    // tx: &mpsc::UnboundedSender<Message>
) -> Option<f32> {
    let mut rating;
    if depth == 0 {
        // send_board(tx, board.clone());
        rating = evaluate_board(board);
        // thread::sleep(Duration::from_millis(100));
    } else {
        let moves = get_all_valid_moves(board);

        rating = f32::NEG_INFINITY;

        for valid_move in moves {
            let (new_board, taken_piece) = apply_move(board, valid_move);

            let eval = alpha_beta_prune(new_board, depth - 1, -beta, -alpha, timeout);
            revert_move(board, valid_move, taken_piece);

            if let Some(eval) = eval {
                rating = f32::max(
                    rating,
                    -eval,
                );
                if rating > beta {
                    break;
                }
            } else {
                // bubble up the None so we cancel the search
                return None;
            }

            alpha = f32::max(alpha, rating);
        }
    }
    if depth > 2 {
        // if our depth is > 2 from the bottom level of the search, we will only visit this
        // level relatively rarely. We should poll the remaining time and figure out if we've
        // already taken too long.
        if Instant::now() > timeout {
            return None;
        }
    }
    Some(rating)
}

pub fn alpha_beta_prune_with_best_move(
    board: &mut Board,
    depth: i8,
    mut best_move: Move,
    mut alpha: f32,
    beta: f32,
    timeout: Instant,
) -> Option<(f32, Move)> {
    let mut rating;
    if depth == 0 {
        rating = evaluate_board(board)
    } else {
        let mut moves: VecDeque<Move> = get_all_valid_moves(board).into();
        // move the best move to the front of the list
        moves.retain(|&x| x != best_move);
        moves.push_front(best_move);

        rating = f32::NEG_INFINITY;

        for valid_move in moves {
            let (new_board, taken_piece) = apply_move(board, valid_move);

            let eval = alpha_beta_prune(new_board, depth - 1, -beta, -alpha, timeout);
            revert_move(board, valid_move, taken_piece);

            if let Some(eval) = eval {
                rating = f32::max(
                    -eval,
                    rating,
                );
                if rating > beta {
                    break;
                }
                if rating > alpha {
                    best_move = valid_move;
                    alpha = rating;
                }
            } else {
                return None;
            }
        }
    }
    Some((rating, best_move))
}

// pub fn iterative_deepening(board: &mut Board, max_depth: i8, tx: &mpsc::UnboundedSender<Message>) -> Move {
pub fn iterative_deepening(board: &mut Board, max_depth: i8, timeout_ms: u64) -> Move {
    let moves = get_all_valid_moves(board);
    let mut best_move = moves[0];

    // setup timer
    let end: Instant = Instant::now() + Duration::from_millis(timeout_ms);


    for depth in 0..(max_depth + 1) {
        dbg!(depth);
        if let Some((_new_rating, new_best_move)) = alpha_beta_prune_with_best_move(
            board,
            depth,
            best_move,
            f32::NEG_INFINITY,
            f32::INFINITY,
            end
        ) {
            best_move = new_best_move;
        } else {
            // we've timed out, and need to pass whatever has been calculated
            // already
            return best_move
        }
    }
    best_move
}

pub fn make_a_move(board: &mut Board, timeout_ms: u64) -> Move {
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
    iterative_deepening(board, 20, timeout_ms)
}
