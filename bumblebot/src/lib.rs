use std::{thread, time::Duration, collections::HashMap};

use crate::bot_mind::*;
pub mod bot_mind;
use api::OutgoingMessage;
use hexchesscore::{Board, Hexagon, Piece, Color};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::ws::Message;

pub fn setup_test_boards() -> (Board, Board) {
    let mut board = Board::setup_default_board();

    board.occupied_squares.insert(
        Hexagon::new("B4").unwrap(),
        Piece {
            piece_type: hexchesscore::PieceType::Queen,
            color: Color::Black,
        },
    );

    let mut board2 = Board::setup_default_board();
    board2.occupied_squares.clear();

    board2.occupied_squares.insert(
        Hexagon::new("B4").unwrap(),
        Piece {
            piece_type: hexchesscore::PieceType::Queen,
            color: Color::Black,
        },
    );
    board2.occupied_squares.insert(
        Hexagon::new("C2").unwrap(),
        Piece {
            piece_type: hexchesscore::PieceType::Rook,
            color: Color::Black,
        },
    );
    board2.occupied_squares.insert(
        Hexagon::new("A3").unwrap(),
        Piece {
            piece_type: hexchesscore::PieceType::King,
            color: Color::Black,
        },
    );
    board2.occupied_squares.insert(
        Hexagon::new("D3").unwrap(),
        Piece {
            piece_type: hexchesscore::PieceType::King,
            color: Color::White,
        },
    );
    (board, board2)
}


#[cfg(test)]
mod tests {
    use hexchesscore::{get_all_valid_moves, Board, Color, Hexagon, Piece};

    use super::*;

    #[test]
    fn test_board_evaluation() {
        let mut board = Board::setup_default_board();

        // clear the board
        board.occupied_squares.clear();

        // add a single white queen
        board.occupied_squares.insert(
            Hexagon::new("A5").unwrap(),
            Piece {
                piece_type: hexchesscore::PieceType::Queen,
                color: Color::White,
            },
        );
        assert!(evaluate_board(&board) == 9.0);

        // add a second white queen
        board.occupied_squares.insert(
            Hexagon::new("B5").unwrap(),
            Piece {
                piece_type: hexchesscore::PieceType::Queen,
                color: Color::White,
            },
        );
        assert!(evaluate_board(&board) == 18.0);

        // now add a black queen
        board.occupied_squares.insert(
            Hexagon::new("C3").unwrap(),
            Piece {
                piece_type: hexchesscore::PieceType::Queen,
                color: Color::Black,
            },
        );
        assert!(evaluate_board(&board) == 9.0);
    }

    #[test]
    fn test_alpha_beta_pruning_matches_minimax() {
        let (board, board2) = setup_test_boards();

        for mut board in [board, board2] {
            for depth in [0, 1, 2, 3] {
                for color in [Color::Black, Color::White] {
                    board.current_player = color;
                    dbg!(color);
                    dbg!(board.current_player);
                    let a_eval =
                        alpha_beta_prune(&mut board, depth, f32::NEG_INFINITY, f32::INFINITY);
                    dbg!(board.current_player);
                    let nega_eval = negamax(&mut board, depth);
                    dbg!(depth);
                    dbg!(a_eval);
                    dbg!(nega_eval);
                    // dbg!(iid);
                    assert!(a_eval == nega_eval);
                    // assert!(a_eval == iid);
                }
            }
        }
    }
}
