


pub mod bot_mind;
pub mod random_bot;
pub mod random_bot2;


use hexchesscore::{Board, Color, Hexagon, Piece};




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
    use std::{time::Instant, fs::File, io::Write};

    use hexchesscore::{get_all_valid_moves, Board, Color, Hexagon, Piece};
    use random_bot::get_samples;

    use crate::random_bot2::SearchTree;

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
                    let a_eval = alpha_beta_prune(
                        &mut board,
                        depth,
                        f32::NEG_INFINITY,
                        f32::INFINITY,
                        Instant::now() + Duration::from_millis(100000),
                    )
                    .expect("timed out!");
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
    #[test]
    fn test_random_sampling() {
        let n = 1000;
        let num_moves = 20;
        let num_samples = 50;
        let samples: Vec<Vec<u16>> = (0..n)
            .map(|x| {
                get_samples(
                    num_samples as f32,
                    (0..num_moves).map(|x| x as f32).collect(),
                )
            })
            .collect();
        for sample in &samples {
            let total: u16 = sample.iter().sum();
            assert!(total == num_samples as u16);
        }
        let stats = samples.iter().fold(vec![0; num_moves], |acc, s| {
            acc.iter().zip(s).map(|(x, y)| x + y).collect()
        });
        dbg!(&stats);
    }

    #[test]
    fn test_tree_search() {
        let mut board = Board::setup_default_board();
        random_bot::make_a_move(&mut board, 100000);
    }

    #[test]
    fn test_new_tree_search() {
        let mut board = Board::setup_default_board();
        let mut tree = SearchTree::new();
            for i in 0..100{
                let out = random_bot2::tree_search(&mut board, &mut tree);
                dbg!(out);
            }
    }

        #[test]
        fn test_tree_search_finds_checkmate() {
            let mut board = Board::new();
            board.occupied_squares.insert(
                Hexagon::new("D2").unwrap(),
                Piece {
                piece_type: hexchesscore::PieceType::Queen,
                color: Color::Black,
            },
        );
        board.occupied_squares.insert(
            Hexagon::new("C6").unwrap(),
            Piece {
                piece_type: hexchesscore::PieceType::King,
                color: Color::Black,
            },
        );
        board.occupied_squares.insert(
            Hexagon::new("A4").unwrap(),
            Piece {
                piece_type: hexchesscore::PieceType::King,
                color: Color::White,
            },
        );
        board.current_player = Color::Black;
        output_board_representation(&board);
        
        dbg!(random_bot::make_a_move(&mut board, 1000));
    }
    
    fn output_board_representation(board: &Board) {
        let mut f = File::create("../server/debug/board.json").expect("Couldn't open file");

        write!(
            f,
            "{}",
            serde_json::to_string(&board).expect("couldn't serialize board")
        )
        .expect("couldn't write to disk");
    }
}
