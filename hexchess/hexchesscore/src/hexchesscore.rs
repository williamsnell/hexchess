use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

pub fn rank_char_to_int(rank: char) -> Option<u8> {
    match rank {
        'a' => Some(0),
        'b' => Some(1),
        'c' => Some(2),
        'd' => Some(3),
        'e' => Some(4),
        'f' => Some(5),
        'g' => Some(6),
        'h' => Some(7),
        'i' => Some(8),
        'k' => Some(9),
        'l' => Some(10),
        _ => None
    }
}

pub fn rank_int_to_char(rank: u8) -> Option<char> {
    match rank {
        0 => Some('a'),
        1 => Some('b'),
        2 => Some('c'),
        3 => Some('d'),
        4 => Some('e'),
        5 => Some('f'),
        6 => Some('g'),
        7 => Some('h'),
        8 => Some('i'),
        9 => Some('k'),
        10 => Some('l'),
        _ => None,
    }
}


#[derive(Eq, Hash, PartialEq, Debug, Ord, PartialOrd, Serialize, Deserialize, Clone, Copy)]
pub struct Hexagon {
    pub rank: u8,
    pub file: u8,
}

impl Hexagon {
    pub fn new(square: &str) -> Option<Hexagon> {
        let char_vec: Vec<char> = square.to_lowercase().chars().collect();
        match char_vec[..] {
            ['j', _] => None, // j is annoyingly not a valid rank
            [rank @ 'a'..='l', file @ '1'..='9'] => Some(Hexagon {
                rank: rank_char_to_int(rank).expect("Invalid rank entered"),
                file: (file.to_digit(10).unwrap() - 1) as u8,
            }),
            [rank @ 'a'..='l', '1', file2 @ '0'..='1'] => Some(Hexagon {
                rank: rank_char_to_int(rank).expect("Invalid rank entered"),
                file: (file2.to_digit(10).unwrap() + 9) as u8,
            }),
            _ => None,
        }
    }
}

pub struct Movement {
    origin: Hexagon,
    destination: Hexagon,
}

#[derive(Debug)]
pub struct Board {
    pub occupied_squares: HashMap<Hexagon, Piece>,
}

pub fn validate_move(movement: Movement, board: Board) -> Option<Board> {
    if true {
        Some(board)
    } else {
        None
    }
}
