pub use crate::hexchesscore::*;

pub mod hexchesscore;
pub mod moves;
pub mod board_representations;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::File, io::Write};

    use super::*;

    fn setup_default_board() -> Board {
        let mut default_board = Board {
            occupied_squares: HashMap::new(),
            en_passant: None,
            current_player: Color::White
        };

        default_board.occupied_squares.insert(
            Hexagon::new("A1").unwrap(),
           Piece {
                piece_type: PieceType::Pawn,
                color: Color::White,         },
        );

        default_board
    }
    
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
        assert_eq!(Hexagon::new("F8"), None);
    }

    #[test]
    fn test_check_for_mates_detects_stalemate() {
        let mut board = Board::new();

        board.occupied_squares.insert(Hexagon::new("A4").unwrap(), Piece { piece_type: PieceType::King, color: Color::White });

        board.occupied_squares.insert(Hexagon::new("C4").unwrap(), Piece { piece_type: PieceType::King, color: Color::Black });
  
        board.occupied_squares.insert(Hexagon::new("C7").unwrap(), Piece { piece_type: PieceType::Rook, color: Color::Black });

        output_board_representation(&board);
        
        assert!(matches!(check_for_mates(&mut board).unwrap(), Mate::Stalemate));
    }
    #[test]
    fn test_check_for_mates_detects_checkmate() {
        let mut board = Board::new();

        board.occupied_squares.insert(Hexagon::new("A4").unwrap(), Piece { piece_type: PieceType::King, color: Color::White });

        board.occupied_squares.insert(Hexagon::new("C4").unwrap(), Piece { piece_type: PieceType::King, color: Color::Black });
  
        board.occupied_squares.insert(Hexagon::new("A6").unwrap(), Piece { piece_type: PieceType::Rook, color: Color::Black });

        output_board_representation(&board);
        
        assert!(matches!(check_for_mates(&mut board).unwrap(), Mate::Checkmate));
    }

    #[test]
    fn test_check_for_mates_detects_nothing() {
        let mut board = Board::new();

        board.occupied_squares.insert(Hexagon::new("A4").unwrap(), Piece { piece_type: PieceType::King, color: Color::White });

        board.occupied_squares.insert(Hexagon::new("C4").unwrap(), Piece { piece_type: PieceType::King, color: Color::Black });
  
        board.occupied_squares.insert(Hexagon::new("C8").unwrap(), Piece { piece_type: PieceType::Rook, color: Color::Black });

        output_board_representation(&board);
        
        assert!(check_for_mates(&mut board).is_none());
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

    fn test_invalid_move() {}
}
