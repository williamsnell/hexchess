pub use crate::hexchesscore::*;
use std::collections::HashMap;

pub mod hexchesscore;
pub mod moves;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_default_board() -> Board {
        let mut default_board = Board {
            occupied_squares: HashMap::new(),
        };

        default_board.occupied_squares.insert(
            Hexagon::new("A1").unwrap(),
            Piece {
                piece_type: PieceType::Pawn,
                color: Color::White,
            },
        );

        default_board
    }

    // #[test]
    // fn test_valid_move() {
    //     let result = validate_move(Movement(Hexagon("F", 5), Hexagon("F", 6)), default_board);
    // }
    #[test]
    fn test_new_hex() {
        assert_eq!(Hexagon::new("F8").unwrap().rank, hexchesscore::rank_char_to_int('f').unwrap());
        assert_eq!(Hexagon::new("F8").unwrap().file, 8);
        assert_eq!(
            Hexagon {
                rank: 1,
                file: 11
            },
            Hexagon::new("A11").unwrap()
        );
        assert_eq!(Hexagon::new("M8"), None);
        assert_eq!(Hexagon::new("A13"), None);
        assert_eq!(Hexagon::new(" F8 "), None);
        assert_eq!(Hexagon::new("F8 "), None)
    }

    #[test]
    fn test_rook_moves() {
        let starting_position = Hexagon::new("F6").unwrap();
        let mut valid_moves: Vec<Hexagon> = Vec::from([
            "a1", "b2", "c3", "d4", "e5", "e6", "d6", "c6", "b6", "a6", "f1", "f2", "f3", "f4",
            "f5", "f7", "f8", "f9", "f10", "f11", "g5", "h4", "i3", "k2", "l1", "g6", "h6", "i6",
            "k6", "l6",
        ]
        .map(|x| Hexagon::new(x).unwrap()));

        fn eq_lists_inplace<T>(a: &mut [T], b: &mut [T]) -> bool
        where
            T: PartialEq + Ord + std::fmt::Debug,
        {
            a.sort();
            b.sort();

            println!("{:?}", a);
            println!();
            println!("{:?}", b);

            a == b
        }

        let mut output_moves: Vec<Hexagon> = moves::RookMoves::new(starting_position).collect();
        
        assert!(eq_lists_inplace(&mut output_moves, &mut valid_moves));



        // [Hexagon::new(square) for square in valid_moves];
    }

    fn test_invalid_move() {}
}
