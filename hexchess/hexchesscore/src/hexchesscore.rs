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
        'a' => Some(1),
        'b' => Some(2),
        'c' => Some(3),
        'd' => Some(4),
        'e' => Some(5),
        'f' => Some(6),
        'g' => Some(7),
        'h' => Some(8),
        'i' => Some(9),
        'k' => Some(10),
        'l' => Some(11),
        _ => None,
    }
}

pub fn rank_int_to_char(rank: u8) -> Option<char> {
    match rank {
        1 => Some('a'),
        2 => Some('b'),
        3 => Some('c'),
        4 => Some('d'),
        5 => Some('e'),
        6 => Some('f'),
        7 => Some('g'),
        8 => Some('h'),
        9 => Some('i'),
        10 => Some('k'),
        11 => Some('l'),
        _ => None,
    }
}


#[derive(Eq, Hash, PartialEq, Debug, Ord, PartialOrd, Serialize, Deserialize)]
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
                file: file.to_digit(10).unwrap() as u8,
            }),
            [rank @ 'a'..='l', '1', file2 @ '0'..='1'] => Some(Hexagon {
                rank: rank_char_to_int(rank).expect("Invalid rank entered"),
                file: (file2.to_digit(10).unwrap() + 10) as u8,
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
