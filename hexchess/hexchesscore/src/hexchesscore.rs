use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::{fs, path::PathBuf};

use crate::moves::{self, KnightMoves, SlidingMoves};

#[derive(Debug, Serialize, Deserialize)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
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
        _ => None,
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
        S: Serializer,
    {
        let rank_char = rank_int_to_char(self.rank).unwrap();
        let file = self.file + 1;
        serializer.serialize_str(format!("{rank_char}{file}").as_str())
    }
}

struct HexagonVisitor;

impl<'de> Visitor<'de> for HexagonVisitor {
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
            None => Err(E::custom("Invalid Hexagon Chess Cell")),
        }
    }
}

impl<'de> Deserialize<'de> for Hexagon {
    fn deserialize<D>(deserializer: D) -> Result<Hexagon, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HexagonVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
    pub occupied_squares: HashMap<Hexagon, Piece>,
    pub en_passant: Option<Hexagon>,
    pub current_player: Color,
}

impl Board {
    pub fn setup_default_board() -> Board {
        let path = PathBuf::from("./server_files/starting_moves.json");
        let data = fs::read_to_string(path).expect("unable to read file");
        let moves: HashMap<Hexagon, Piece> =
            serde_json::from_str(&data).expect("Invalid JSON format");
        Board {
            occupied_squares: moves,
            en_passant: None,
            current_player: Color::White,
        }
    }
}

pub fn is_king_in_check(board: Board) -> bool {
    true
}

// pub fn get_valid_moves_without_checks(hexagon: Hexagon, piece: Piece, board: Board) -> Vec<Hexagon> {
//     // checking for check involves checking all the valid moves for attacking pieces,
//     // so break out the functionality here
// }

fn get_blocking_sliding_moves(
    mut moves: SlidingMoves,
    piece: &Piece,
    board: &Board,
) -> Vec<Hexagon> {
    let mut valid_moves = Vec::<Hexagon>::new();

    while let Some(hexagon) = moves.next() {
        if let Some(occupied_hex) = board.occupied_squares.get(&hexagon) {
            if occupied_hex.color != piece.color {
                valid_moves.push(hexagon);
            };
            // regardless of if the color matched or not, this arm is now blocked by
            // the piece
            moves.drop_arm();
        } else {
            valid_moves.push(hexagon);
        }
    }

    valid_moves
}

fn get_valid_knight_moves(moves: moves::KnightMoves, piece: &Piece, board: &Board) -> Vec<Hexagon> {
    let mut valid_moves = Vec::<Hexagon>::new();
    for hex in moves {
        if let Some(occupied_hex) = board.occupied_squares.get(&hex) {
            if occupied_hex.color != piece.color {
                valid_moves.push(hex);
            };
        } else {
            valid_moves.push(hex);
        }
    }
    valid_moves
}

pub fn get_valid_moves(
    hexagon: &Hexagon,
    piece: &Piece,
    board: &Board,
) -> (Vec<Hexagon>, Option<Hexagon>) {
    // get valid pieces
    // check for friendly pieces blocking stuff
    // check for enemy pieces allowing captures
    let (valid_moves, double_jump) = match piece.piece_type {
        PieceType::Rook | PieceType::Queen | PieceType::Bishop | PieceType::King => (
            get_blocking_sliding_moves(SlidingMoves::new(&hexagon, &piece), piece, board),
            Option::<Hexagon>::None,
        ),
        PieceType::Pawn => moves::pawn_moves(hexagon, &piece.color, board),
        PieceType::Knight => (
            get_valid_knight_moves(KnightMoves::new(hexagon), piece, board),
            None,
        ),
    };

    // validate the king is not in check for any of the moves

    (valid_moves, double_jump)
}

pub enum HexChessError {
    FailedToRegisterMove,
    NotYourTurn,
}

pub fn register_move(
    start_hexagon: &Hexagon,
    final_hexagon: &Hexagon,
    board: &mut Board,
    double_jump: Option<Hexagon>,
) -> Result<Color, HexChessError> {
    // If succesful, return the color of the player whose turn
    // it will now be.
    let moving_color = board.occupied_squares.get(&start_hexagon).unwrap().color;
    let valid_player = board.current_player;
    // make sure the right player is trying to move
    if !(moving_color == valid_player) {
        return Err(HexChessError::NotYourTurn);
    }

    let new_color = match moving_color {
        Color::White => Color::Black,
        Color::Black => Color::White,
    };

    // try remove the moving piece from the old hex
    match board.occupied_squares.remove(&start_hexagon) {
        Some(piece) => {
            // try insert the moving piece in the new hex
            match board.occupied_squares.insert(*final_hexagon, piece) {
                Some(_) => {}
                None => {
                    // if we have just completed an en-passant,
                    // we need to remove the pawn one hexagon
                    // up or down
                    holy_hell(board, final_hexagon, valid_player);
                }
            };

            // If a pawn has double jumped, then we need to register it as
            // the latest en-passant. On the other hand, if there was no
            // double jump, the latest en-passant will be None
            match double_jump {
                Some(hex) => {
                    if final_hexagon == &hex {
                        board.en_passant = double_jump;
                    }
                }
                _ => board.en_passant = None,
            }
            
            board.current_player = new_color;
            Ok(new_color)
        }
        None => Err(HexChessError::FailedToRegisterMove),
    }
}

fn holy_hell(board: &mut Board, final_hexagon: &Hexagon, valid_player: Color) {
    if board.en_passant.is_some() {
        let mut new_hex = final_hexagon.clone();
        let actual_pawn_file = match valid_player {
            Color::White => new_hex.file - 1,
            Color::Black => new_hex.file + 1,
        };
        new_hex.file = actual_pawn_file;
        if new_hex == board.en_passant.unwrap() {
            board.occupied_squares.remove(&new_hex).unwrap();
        }
    }
}
