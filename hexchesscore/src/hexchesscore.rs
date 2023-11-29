use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::{fs, path::PathBuf};

use crate::moves::{self, get_rank_length, KnightMoves, SlidingMoves};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum PieceType {
 Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn invert(&self) -> Color {
        match &self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
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

#[derive(Eq, Hash, PartialEq, Ord, PartialOrd, Clone, Copy)]
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

impl fmt::Debug for Hexagon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize(serde_json::value::Serializer).unwrap())
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

const DEFAULT_BOARD: &str = include_str!("./starting_moves.json");

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Board {
    pub occupied_squares: HashMap<Hexagon, Piece>,
    pub en_passant: Option<Hexagon>,
    pub current_player: Color,
}


impl Board {
    pub fn setup_default_board() -> Board {
        let moves: HashMap<Hexagon, Piece> =
            serde_json::from_str(&DEFAULT_BOARD).expect("Invalid JSON format");
        Board {
            occupied_squares: moves,
            en_passant: None,
            current_player: Color::White,
        }
    }
    pub fn new() -> Board {
        Board {
            occupied_squares: HashMap::<Hexagon, Piece>::new(),
            en_passant: None,
            current_player: Color::White,
        }
    }
}

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

fn get_valid_moves_without_checks(
    hexagon: &Hexagon,
    piece: &Piece,
    board: &Board,
) -> (Vec<Hexagon>, Option<Hexagon>, Vec<Hexagon>) {
    // get valid pieces
    // check for friendly pieces blocking stuff
    // check for enemy pieces allowing captures
    let (valid_moves, double_jump, promotion_moves) = match piece.piece_type {
        PieceType::Rook | PieceType::Queen | PieceType::Bishop | PieceType::King => (
            get_blocking_sliding_moves(SlidingMoves::new(&hexagon, &piece), piece, board),
            None,
            Vec::new(),
        ),
        PieceType::Pawn => moves::pawn_moves(hexagon, &piece.color, board),
        PieceType::Knight => (
            get_valid_knight_moves(KnightMoves::new(hexagon), piece, board),
            None,
            Vec::new(),
        ),
    };

    (valid_moves, double_jump, promotion_moves)
}

pub fn get_all_pieces_of_matching_color(color: Color, board: &Board) -> Vec<(Hexagon, Piece)> {
    let mut vec = Vec::<(Hexagon, Piece)>::new();
    for (hex, piece) in &board.occupied_squares {
        if piece.color == color {
            vec.push((*hex, *piece));
        }
    }
    vec
}

fn pieces_can_attack_king(
    enemy_piece_hex: &Hexagon,
    king_pos: &Hexagon,
    enemy_piece: &Piece,
    board: &Board,
) -> bool {
    let mut out = false;
    for enemy_move in get_valid_moves_without_checks(enemy_piece_hex, enemy_piece, board).0 {
        if &enemy_move == king_pos {
            out = true;
            break;
        }
    }
    out
}

pub fn get_attacking_pieces(
    enemy_color: Color,
    board: &mut Board,
    king_pos: &Hexagon,
) -> Option<Vec<(Hexagon, Piece)>> {
    let mut potential_pieces = get_all_pieces_of_matching_color(enemy_color, board);

    potential_pieces.retain(|(enemy_piece_hex, enemy_piece)| {
        pieces_can_attack_king(enemy_piece_hex, king_pos, enemy_piece, board)
    });

    if !potential_pieces.is_empty() {
        Some(potential_pieces)
    } else {
        None
    }
}

pub fn check_moves_for_checks(
    moves: &mut Vec<Hexagon>,
    hexagon: &Hexagon,
    piece: &Piece,
    board: &mut Board,
) {
    let piece_is_king = matches!(piece.piece_type, PieceType::King);

    let inverted_board: HashMap<Piece, Hexagon> = board
        .occupied_squares
        .iter()
        .map(|(k, v)| (v.clone(), k.clone()))
        .collect();

    // remove the moving piece from the board
    board.occupied_squares.remove(hexagon);

    if piece_is_king {
        // if the piece is the king, see if there are any attacking pieces for each possible king move
        moves.retain(|king_move| {
            let king_is_safe;

            // before we overwrite the board, check what is currently there
            let existing_piece = board.occupied_squares.remove(king_move);
            board.occupied_squares.insert(*king_move, *piece);

            match get_attacking_pieces(piece.color.invert(), board, king_move) {
                Some(_) => king_is_safe = false,
                None => king_is_safe = true,
            }

            board.occupied_squares.remove(king_move);
            if existing_piece.is_some() {
                board
                    .occupied_squares
                    .insert(*king_move, existing_piece.unwrap());
            }
            king_is_safe
        });
    } else {
        let king_pos = inverted_board
            .get(&Piece {
                piece_type: PieceType::King,
                color: piece.color,
            })
            .expect("Couldn't find the king?!?");

        // otherwise, first, see if any pieces can attack the king if the piece wasn't there.
        // then, go through all the possible attackers
        // We have to manually disable en-passant at this point
        let en_passant = board.en_passant.clone();
        board.en_passant = None;

        if let Some(attackers) = get_attacking_pieces(piece.color.invert(), board, king_pos) {
            // if they can, store all the attacking pieces
            // that can reach the king

            // for each move in the list,
            // evaluate if any of these pieces can still attack the king
            moves.retain(|hex| {
                let mut king_is_safe = true;
                // add the moving piece into its potential board position
                let existing_piece = board.occupied_squares.remove(hex);
                board.occupied_squares.insert(*hex, *piece);

                // check for potential attacks
                for (enemy_hex, enemy_piece) in &attackers {
                    // if the piece can't take or block the attacker, the king isn't safe in this situation
                    if (hex != enemy_hex)
                        & pieces_can_attack_king(enemy_hex, king_pos, enemy_piece, board)
                    {
                        king_is_safe = false;
                        break;
                    }
                }
                // clean up by removing the piece inserted into the board
                board.occupied_squares.remove(hex);
                if existing_piece.is_some() {
                    board.occupied_squares.insert(*hex, existing_piece.unwrap());
                }

                // if the king is safe, we can keep this move
                king_is_safe
            });
        }
        board.en_passant = en_passant;
    }
    // add the piece back to the board
    board.occupied_squares.insert(*hexagon, *piece);
}

/// From a given starting hexagon, give all the valid
/// moves that a piece (which might be located there) has.
/// If a piece isn't located there, there will be no valid
/// moves.
pub fn get_valid_moves(
    hexagon: &Hexagon,
    board: &mut Board,
) -> (Vec<Hexagon>, Option<Hexagon>, Vec<Hexagon>) {
    let mut valid_moves;
    let double_jump;
    let promotion_moves;
    // check that the piece
    if let Some(piece) = board.occupied_squares.get(&hexagon).cloned() {
        (valid_moves, double_jump, promotion_moves) =
            get_valid_moves_without_checks(hexagon, &piece, board);
        // validate the king is not in check for any of the moves
        // -> this in-place mutates the valid_moves vec
        check_moves_for_checks(&mut valid_moves, hexagon, &piece, board);
    } else {
        (valid_moves, double_jump, promotion_moves) =
            (Vec::<Hexagon>::new(), None, Vec::<Hexagon>::new());
    }
    (valid_moves, double_jump, promotion_moves)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Move {
    pub start_hex: Hexagon,
    pub final_hex: Hexagon,
    pub final_piece: PieceType,
}

pub fn get_all_valid_moves(board: &mut Board) -> Vec<Move> {
    let mut moves = Vec::<Move>::new();
    for (starting_hex, moving_piece) in
        get_all_pieces_of_matching_color(board.current_player, board)
    {
        let (valid_moves, _double_jump, promotion_moves) = get_valid_moves(&starting_hex, board);
        for final_hex in valid_moves {
            // check if we're trying to promote
            if promotion_moves.contains(&final_hex)
                && matches!(moving_piece.piece_type, PieceType::Pawn)
            {
                for piece_type in [
                    PieceType::Bishop,
                    PieceType::Knight,
                    PieceType::Rook,
                    PieceType::Queen,
                ] {
                    moves.push(Move {
                        start_hex: starting_hex,
                        final_hex: final_hex,
                        final_piece: piece_type,
                    })
                }
            } else {
                moves.push(Move {
                    start_hex: starting_hex,
                    final_hex: final_hex,
                    final_piece: moving_piece.piece_type,
                })
            }
        }
    }
    moves
}

#[derive(Debug, Clone, Copy)]
pub enum Mate {
    Checkmate,
    Stalemate,
}

pub fn check_for_mates(board: &mut Board) -> Option<Mate> {
    let current_player_color = board.current_player;

    // Get all pieces of the current player
    let player_pieces = get_all_pieces_of_matching_color(current_player_color, board);

    // Check if any of the player's pieces have a valid move
    for (start_hexagon, _piece) in player_pieces {
        let (valid_moves, _, _) = get_valid_moves(&start_hexagon, board);

        // If any valid move exists, the player is not in checkmate
                if !valid_moves.is_empty() {
            return None;
        }
    }

    // The player's king is in check if there are attacking pieces
    let king_hex = get_all_pieces_of_matching_color(current_player_color, board)
        .into_iter()
        .find(|(_, piece)| piece.piece_type == PieceType::King)
        .map(|(hex, _)| hex).expect("couldn't find the king");

    if let Some(attacking_pieces) =
        get_attacking_pieces(current_player_color.invert(), board, &king_hex)
    {
        // get_attacking_pieces will only return Some if there are attacking pieces. Hence, 
        // the king has no valid moves and is under attack, indicating a checkmate
        return Some(Mate::Checkmate);
    } else {
        return Some(Mate::Stalemate);
    }
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
    promotion_moves: Vec<Hexagon>,
    promotion_choice: Option<PieceType>,
) -> Result<Color, HexChessError> {
    // If succesful, return the color of the player whose turn
    // it will now be.
    let moving_color = board.occupied_squares.get(&start_hexagon).unwrap().color;
    let valid_player = board.current_player;
    // make sure the right player is trying to move
    if !(moving_color == valid_player) {
        return Err(HexChessError::NotYourTurn);
    }

    // try remove the moving piece from the old hex
    match board.occupied_squares.remove(&start_hexagon) {
        Some(piece) => {
            // check if we're trying to promote
            if matches!(piece.piece_type, PieceType::Pawn)
                && promotion_moves.contains(final_hexagon)
            {
                if promotion_choice.is_some()
                    && !matches!(promotion_choice.unwrap(), PieceType::Pawn | PieceType::King)
                {
                    // make sure a promotion choice has been selected and it isn't a pawn
                    // or a king
                    let newly_created_piece = Piece {
                        piece_type: promotion_choice.unwrap(),
                        color: moving_color,
                    };
                    board
                        .occupied_squares
                        .insert(*final_hexagon, newly_created_piece);
                } else {
                    board.occupied_squares.insert(*start_hexagon, piece);
                    return Err(HexChessError::FailedToRegisterMove);
                }
            } else {
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
            }

            board.current_player = moving_color.invert();
            Ok(moving_color.invert())
        }
        None => Err(HexChessError::FailedToRegisterMove),
    }
}

pub fn final_hex_is_valid(final_hexagon: &Hexagon, valid_player: Color) -> bool {
    match valid_player {
        Color::White => final_hexagon.file > 0,
        Color::Black => final_hexagon.file <= get_rank_length(final_hexagon.rank).unwrap(),
    }
}

fn holy_hell(board: &mut Board, final_hexagon: &Hexagon, valid_player: Color) {
    if board.en_passant.is_some() {
        if board
            .occupied_squares
            .get(&board.en_passant.unwrap())
            .is_some()
        {
            if (board
                .occupied_squares
                .get(&board.en_passant.unwrap())
                .unwrap()
                .color
                != valid_player)
                & final_hex_is_valid(final_hexagon, valid_player)
            {
                let new_hex = convert_en_passant_to_virtual_pawn(final_hexagon, valid_player);
                if new_hex == board.en_passant.unwrap() {
                    board.occupied_squares.remove(&new_hex).unwrap();
                }
            }
        }
    }
}

pub fn convert_en_passant_to_virtual_pawn(final_hexagon: &Hexagon, valid_player: Color) -> Hexagon {
    let mut new_hex = final_hexagon.clone();
    let actual_pawn_file = match valid_player {
        Color::White => new_hex.file - 1,
        Color::Black => new_hex.file + 1,
    };
    new_hex.file = actual_pawn_file;
    new_hex
}


pub fn apply_move(board: &mut Board, movement: Move) -> (&mut Board, Option<Piece>) {
    // this function assumes the move is legal. The legality checking
    // should have already happened in the move generation
    let moving_piece = board
        .occupied_squares
        .remove(&movement.start_hex)
        .expect("Piece wasn't present at start hex");
    let taken_piece = board
        .occupied_squares
        .insert(movement.final_hex, moving_piece);
    board.current_player = board.current_player.invert();
    (board, taken_piece)
}

pub fn revert_move(
    board: &mut Board,
    movement: Move,
    taken_piece: Option<Piece>,
) -> (&mut Board, Option<Piece>) {
    // this function assumes the move is legal. The legality checking
    // should have already happened in the move generation
    let moving_piece = board
        .occupied_squares
        .remove(&movement.final_hex)
        .expect("Piece wasn't present at final hex");
    board
        .occupied_squares
        .insert(movement.start_hex, moving_piece);
    if let Some(taken_piece) = taken_piece {
        board
            .occupied_squares
            .insert(movement.final_hex, taken_piece);
    }
    board.current_player = board.current_player.invert();
    (board, taken_piece)
}
