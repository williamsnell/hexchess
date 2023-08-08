use std::fmt;
use std::{fs, path::PathBuf};
use std::collections::HashMap;
use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize, Serializer, Deserializer};

#[derive(Debug, Serialize, Deserialize)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Serialize, Deserialize)]
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


#[derive(Eq, Hash, PartialEq, Debug, Ord, PartialOrd, Clone, Copy)]
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

impl Serialize for Hexagon {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
    where
        S: Serializer
    {
        let rank_char = rank_int_to_char(self.rank).unwrap();
        let file = self.file;
        serializer.serialize_str(format!("{rank_char}{file}").as_str())
    }
}

struct HexagonVisitor;

impl <'de> Visitor<'de> for HexagonVisitor {
    type Value = Hexagon;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A valid letter/number hex chess move, e.g. A5, F10, etc.")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where 
        E: de::Error,
    {
        match Hexagon::new(value) {
            Some(val) => Ok(val),
            None => Err(E::custom("Invalid Hexagon Chess Cell"))
        }
    }
}

impl <'de> Deserialize<'de> for Hexagon {
    fn deserialize<D>(deserializer: D) -> Result<Hexagon, D::Error> 
    where
        D: Deserializer<'de>
    {
        deserializer.deserialize_str(HexagonVisitor)
    }
}


pub struct Movement {
    origin: Hexagon,
    destination: Hexagon,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
    occupied_squares: HashMap<Hexagon, Piece>,
}

impl Board {
    fn pieces(&mut self) -> &mut HashMap<Hexagon, Piece> {
        &mut self.occupied_squares
    }
}

impl Board{
    pub fn setup_default_board() -> Board {
        let path = PathBuf::from("./server_files/starting_moves.json");
        let data = fs::read_to_string(path).expect("unable to read file");
        let moves: serde_json::Value = serde_json::from_str(&data).expect("Invalid JSON format");

        // println!("{:?}", moves);
        let occupied_squares = HashMap::<Hexagon, Piece>::new();

        let mut b = Board{
            occupied_squares: HashMap::<Hexagon, Piece>::new()
        };

        let mut pieces = b.pieces();

        pieces.insert(Hexagon::new("A6").unwrap(), 
            Piece { piece_type: PieceType::Knight, color: Color::Black });

        println!("{:?}", serde_json::to_string(&b));
        b
    }
}

pub fn validate_move(movement: Movement, board: Board) -> Option<Board> {
    if true {
        Some(board)
    } else {
        None
    }
}
