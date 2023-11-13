use std::collections::HashMap;

use crate::{Board, Color, Hexagon, Piece, PieceType};
use bitvec::prelude::*;

const RANK_LENGTH: [u32; 11] = [6, 7, 8, 9, 10, 11, 10, 9, 8, 7, 6];
const CUMULATIVE_HEXES: [u32; 11] = [0, 6, 13, 21, 30, 40, 51, 61, 70, 78, 85];

type SubBoard = u128;

#[derive(PartialEq, Debug)]
pub struct BitBoard {
    white_king: SubBoard,
    white_queen: SubBoard,
    white_rook: SubBoard,
    white_bishop: SubBoard,
    white_knight: SubBoard,
    white_pawn: SubBoard,
    black_king: SubBoard,
    black_queen: SubBoard,
    black_rook: SubBoard,
    black_bishop: SubBoard,
    black_knight: SubBoard,
    black_pawn: SubBoard,
    current_player_is_white: bool,
}

impl BitBoard {
    pub fn new() -> Self {
        BitBoard {
            white_king: 0,
            white_queen: 0,
            white_rook: 0,
            white_bishop: 0,
            white_knight: 0,
            white_pawn: 0,
            black_king: 0,
            black_queen: 0,
            black_rook: 0,
            black_bishop: 0,
            black_knight: 0,
            black_pawn: 0,
            current_player_is_white: true,
        }
    }
    pub fn get_subboard(&self, piece: Piece) -> &SubBoard {
        match piece.color {
            Color::White => match piece.piece_type {
                PieceType::King => &self.white_king,
                PieceType::Queen => &self.white_queen,
                PieceType::Rook => &self.white_rook,
                PieceType::Bishop => &self.white_bishop,
                PieceType::Knight => &self.white_knight,
                PieceType::Pawn => &self.white_pawn,
            },
            Color::Black => match piece.piece_type {
                PieceType::King => &self.black_king,
                PieceType::Queen => &self.black_queen,
                PieceType::Rook => &self.black_rook,
                PieceType::Bishop => &self.black_bishop,
                PieceType::Knight => &self.black_knight,
                PieceType::Pawn => &self.black_pawn,
            },
        }
    }
    pub fn set_subboard(&mut self, piece: Piece, val: SubBoard) {
        match piece.color {
            Color::White => match piece.piece_type {
                PieceType::King => self.white_king = val,
                PieceType::Queen => self.white_queen = val,
                PieceType::Rook => self.white_rook = val,
                PieceType::Bishop => self.white_bishop = val,
                PieceType::Knight => self.white_knight = val,
                PieceType::Pawn => self.white_pawn = val,
            },
            Color::Black => match piece.piece_type {
                PieceType::King => self.black_king = val,
                PieceType::Queen => self.black_queen = val,
                PieceType::Rook => self.black_rook = val,
                PieceType::Bishop => self.black_bishop = val,
                PieceType::Knight => self.black_knight = val,
                PieceType::Pawn => self.black_pawn = val,
            },
        }
    }
    /// Convert the Hexagon {rank, file} representation into
    /// a bit mask, where there will be a single 1
    /// bit matching the relevant position in all of the BitBoard
    /// types' bit boards.
    pub fn bit_mask_from_hexagon(hex: Hexagon) -> SubBoard {
        dbg!(&hex);
        let position = CUMULATIVE_HEXES[hex.rank as usize] + hex.file as u32;
        (2 as SubBoard).pow(position)
    }

    pub fn hexagon_from_bit_position(position: u32) -> Option<Hexagon> {
        for (i, offset) in CUMULATIVE_HEXES.iter().enumerate().rev() {
            if position > *offset {
                return Some(Hexagon {
                    rank: (i) as u8,
                    file: (position - offset) as u8,
                });
            }
        }
        None
    }

    pub fn invert_sub_board_at_hex(sub_board: &SubBoard, hex: Hexagon) -> SubBoard {
        let mask = BitBoard::bit_mask_from_hexagon(hex);
        sub_board ^ mask
    }

    // Implicitly assumes the piece is in fact at this position.
    // If that assumption is wrong, it will mistakenly insert
    // a piece at the specified position.
    pub fn insert_piece(&mut self, piece: Piece, hex: Hexagon) {
        let sub_board = self.get_subboard(piece);
        self.set_subboard(piece, BitBoard::invert_sub_board_at_hex(sub_board, hex));
    }

    pub fn from_board(board: &Board) -> Self {
        let mut bitboard = BitBoard::new();
        for (hex, piece) in &board.occupied_squares {
            bitboard.insert_piece(*piece, *hex);
        }
        // TODO: transfer across the double-jumped pawn mappings
        bitboard.current_player_is_white = if board.current_player == Color::White {
            true
        } else {
            false
        };
        bitboard
    }

    pub fn subboard_to_hexes(subboard: SubBoard) -> Vec<Hexagon> {
        let split_int = [subboard as u64, (subboard >> 64) as u64];
        let mut hexes = Vec::<Hexagon>::new();
        for (i, bit) in split_int.view_bits::<Lsb0>().iter().enumerate() {
            if *bit {
                hexes.push(
                    BitBoard::hexagon_from_bit_position(i as u32)
                        .expect("Position was not converted to a valid hexagon"),
                )
            }
        }
        hexes
    }

    pub fn to_board(&self) -> Board {
        let mut board = Board {
            occupied_squares: HashMap::<Hexagon, Piece>::new(),
            en_passant: None,
            current_player: if self.current_player_is_white {Color::White} else {Color::Black},
        };
        for color in [Color::Black, Color::White] {
            for piece_type in [
                PieceType::King,
                PieceType::Queen,
                PieceType::Rook,
                PieceType::Bishop,
                PieceType::Knight,
                PieceType::Pawn,
            ] {
                let piece = Piece {
                    piece_type: piece_type,
                    color: color,
                };
                for hex in BitBoard::subboard_to_hexes(*self.get_subboard(piece)) {
                    board.occupied_squares.insert(hex, piece);
                }
            }
        }
        board
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, fs::File, io::Write};

    use super::*;
    use pretty_assertions::{assert_eq};

    #[test]
    fn test_bit_representation() {
        let val = BitBoard::bit_mask_from_hexagon(Hexagon { rank: 0, file: 1 });
        let val = val ^ BitBoard::bit_mask_from_hexagon(Hexagon { rank: 0, file: 0 });
        let vals = [(val >> 64) as u64, val as u64];
        let arr = vals.view_bits::<Msb0>();
        dbg!(arr);
    }
    #[test]
    fn test_bit_to_hex_to_bit() {
        let board = BitBoard::bit_mask_from_hexagon(Hexagon { rank: 0, file: 1 });
        dbg!(board);
        let hexes = BitBoard::subboard_to_hexes(board);
        dbg!(&hexes);
        assert!(BitBoard::bit_mask_from_hexagon(hexes[0]) == board);
    }

    #[test]
    fn test_bitboard_conversion() {
        let board = Board::setup_default_board();
        let bitboard = BitBoard::from_board(&board);
        let board2 = bitboard.to_board();
        let bitboard2 = BitBoard::from_board(&board2);

        output_board_representation(&board);
        output_board_representation(&board2); 



        assert_eq!(&bitboard, &bitboard2);

        assert_eq!(&board.occupied_squares,&board2.occupied_squares);
    }

    fn output_board_representation(board: &Board) {
        let mut f = File::create("../server/debug/board.json").expect("Couldn't open file");

        write!(f, "{}", serde_json::to_string(&board).expect("couldn't serialize board")).expect("couldn't write to disk");
    }
}
