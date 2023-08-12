use std::{cmp::min, iter::zip, vec::Vec};

use crate::{
    hexchesscore::{convert_en_passant_to_virtual_pawn, Board, Color, Hexagon, Piece},
    PieceType,
};

pub fn get_rank_length(rank: u8) -> Option<u8> {
    match rank {
        0 => Some(6),
        1 => Some(7),
        2 => Some(8),
        3 => Some(9),
        4 => Some(10),
        5 => Some(11),
        6 => Some(10),
        7 => Some(9),
        8 => Some(8),
        9 => Some(7),
        10 => Some(6),
        _ => None,
    }
}

pub fn chess_to_axial_coords(hexagon: &Hexagon) -> (u8, u8) {
    let q: u8 = hexagon.rank;
    let r: u8 = hexagon.file + hexagon.rank + 6
        - get_rank_length(hexagon.rank).unwrap()
        - (if hexagon.rank < 6 {
            0
        } else {
            hexagon.rank - 5
        });
    (q, r)
}

pub fn calc_s(q: u8, r: u8) -> u8 {
    // s = q - r + 5
    5 + q - r
}

pub fn calc_q(r: u8, s: u8) -> u8 {
    s + r - 5
}

pub fn calc_r(q: u8, s: u8) -> u8 {
    5 + q - s
}

pub fn axial_to_chess_coords(q: u8, r: u8) -> Hexagon {
    let rank = q;
    let file = r + get_rank_length(q).unwrap() + (if rank < 6 { 0 } else { rank - 5 }) - rank - 6;
    Hexagon {
        rank: rank,
        file: file,
    }
}

// Sliding moves are all the moves that raycast out from the piece position and are blocked by any piece,
// and can attack the first enemy piece they see
pub struct SlidingMoves {
    move_list: Vec<Vec<Hexagon>>,
}

impl SlidingMoves {
    pub fn new(position: &Hexagon, piece: &Piece) -> SlidingMoves {
        let moves = match piece.piece_type {
            PieceType::Rook => SlidingMoves::new_rook_moves(position),
            PieceType::Bishop => SlidingMoves::new_bishop_moves(position),
            PieceType::Queen => SlidingMoves::new_queen_moves(position),
            PieceType::King => SlidingMoves::new_king_moves(position),
            _ => Vec::<Vec<Hexagon>>::new(),
        };
        SlidingMoves { move_list: moves }
    }

    pub fn new_rook_moves(position: &Hexagon) -> Vec<Vec<Hexagon>> {
        // Get the moves a rook could make, from the current position, assuming
        // no other pieces. These moves spiral out from the rook, so closer hexagons
        // are returned earlier in the result
        // loop through arms
        let (q, r) = chess_to_axial_coords(&position);
        let arm_top: Vec<Hexagon> = (position.file + 1..get_rank_length(position.rank).unwrap())
            .rev()
            .map(|x| Hexagon {
                rank: position.rank,
                file: x,
            })
            .collect();

        let s = calc_s(q, r);
        let arm_bottom: Vec<Hexagon> = (0..position.file)
            .map(|x| Hexagon {
                rank: position.rank,
                file: x,
            })
            .collect();

        let arm_top_left: Vec<Hexagon> = (0..min(q, s))
            .map(|x| axial_to_chess_coords(if q > s { calc_q(r, x) } else { x }, r))
            .collect();

        let arm_bottom_right: Vec<Hexagon> = (min(q, s) + 1..get_rank_length(r).unwrap())
            .rev()
            .map(|x| axial_to_chess_coords(if q > s { calc_q(r, x) } else { x }, r))
            .collect();

        // we use q or r, whichever is smaller
        // we iterate from 0 up to min(q, r)
        // we hold s constant
        let arm_bottom_left: Vec<Hexagon> = (0..min(r, q))
            .map(|x| {
                axial_to_chess_coords(
                    if q > r { calc_q(x, s) } else { x },
                    if q > r { x } else { calc_r(x, s) },
                )
            })
            .collect();

        let arm_top_right: Vec<Hexagon> = (min(r, q) + 1..get_rank_length(s).unwrap())
            .rev()
            .map(|x| {
                axial_to_chess_coords(
                    if q > r { calc_q(x, s) } else { x },
                    if q > r { x } else { calc_r(x, s) },
                )
            })
            .collect();
        vec![
            arm_top,
            arm_bottom,
            arm_top_left,
            arm_bottom_right,
            arm_bottom_left,
            arm_top_right,
        ]
    }

    pub fn new_bishop_moves(position: &Hexagon) -> Vec<Vec<Hexagon>> {
        // Get the moves a bishop could make, from the current position, assuming
        // no other pieces. These moves spiral out from the bishop, so closer hexagons
        // are returned earlier in the result
        let (q, r) = chess_to_axial_coords(&position);
        let s = calc_s(q, r);

        let arm_left: Vec<Hexagon> =
            zip((q % 2..q).step_by(2).rev(), zip((0..r).rev(), (0..s).rev()))
                .map(|(x, (y, _z))| axial_to_chess_coords(x, y))
                .rev()
                .collect();

        let arm_right: Vec<Hexagon> = zip((q + 2..=10).step_by(2), zip(r + 1..=10, s + 1..=10))
            .map(|(x, (y, _z))| axial_to_chess_coords(x, y))
            .rev()
            .collect();

        let arm_down_left: Vec<Hexagon> =
            zip((r % 2..r).step_by(2).rev(), zip((0..q).rev(), s + 1..=10))
                .map(|(x, (y, _z))| axial_to_chess_coords(y, x))
                .rev()
                .collect();

        let arm_up_right: Vec<Hexagon> =
            zip((r + 2..=10).step_by(2), zip(q + 1..=10, (0..s).rev()))
                .map(|(x, (y, _z))| axial_to_chess_coords(y, x))
                .rev()
                .collect();

        let arm_up_left: Vec<Hexagon> =
            zip((s % 2..s).step_by(2).rev(), zip((0..q).rev(), r + 1..=10))
                .map(|(_x, (y, z))| axial_to_chess_coords(y, z))
                .rev()
                .collect();

        let arm_down_right: Vec<Hexagon> =
            zip((s + 2..=10).step_by(2), zip(q + 1..=10, (0..r).rev()))
                .map(|(_x, (y, z))| axial_to_chess_coords(y, z))
                .rev()
                .collect();

        vec![
            arm_left,
            arm_right,
            arm_down_left,
            arm_up_right,
            arm_up_left,
            arm_down_right,
        ]
    }

    pub fn new_queen_moves(position: &Hexagon) -> Vec<Vec<Hexagon>> {
        // Get the moves a bishop could make, from the current position, assuming
        // no other pieces. These moves spiral out from the bishop, so closer hexagons
        // are returned earlier in the result

        let mut queen_moves = SlidingMoves::new_bishop_moves(position);
        let mut rook_moves = SlidingMoves::new_rook_moves(position);
        queen_moves.append(&mut rook_moves);
        queen_moves
    }

    // the king doesn't really belong here, since he doesn't really have
    // arms to drop
    pub fn new_king_moves(position: &Hexagon) -> Vec<Vec<Hexagon>> {
        let king_moves = SlidingMoves::new_queen_moves(position);

        let mut move_list: Vec<Vec<Hexagon>> = Vec::new();

        for mut arm in king_moves {
            if let Some(arm_val) = arm.pop() {
                move_list.push(vec![arm_val]);
            }
        }

        // this is likely very slow, but it's quick to code...
        // get the first move from each of the queen's arms
        move_list
    }

    pub fn drop_arm(&mut self) {
        // drop the current arm of valid sliding moves
        // e.g. if a piece is blocking the remainder of the arm
        self.move_list.pop();
    }
}

impl Iterator for SlidingMoves {
    type Item = Hexagon;
    fn next(&mut self) -> Option<Hexagon> {
        if let Some(moves) = self.move_list.last_mut() {
            if let Some(output) = moves.pop() {
                Some(output)
            } else {
                self.move_list.pop();
                self.next()
            }
        } else {
            None
        }
    }
}

pub struct KnightMoves {
    move_list: Vec<Hexagon>,
}

impl KnightMoves {
    pub fn new(position: &Hexagon) -> KnightMoves {
        let (q, r) = chess_to_axial_coords(&position);
        let s = calc_s(q, r);

        let move_options = [
            //left
            if (q > 1) & (r < 10) & (s > 2) {
                Some(axial_to_chess_coords(q - 2, calc_r(q - 2, s - 3)))
            } else {
                None
            },
            if (q > 2) & (r > 0) & (s > 1) {
                Some(axial_to_chess_coords(q - 3, r - 1))
            } else {
                None
            },
            if (q > 2) & (r > 1) & (s > 0) {
                Some(axial_to_chess_coords(q - 3, r - 2))
            } else {
                None
            },
            if (q > 1) & (r > 2) & (s < 10) {
                Some(axial_to_chess_coords(q - 2, r - 3))
            } else {
                None
            },
            // right
            if (q < 9) & (r > 0) & (s < 8) {
                Some(axial_to_chess_coords(q + 2, calc_r(q + 2, s + 3)))
            } else {
                None
            },
            if (q < 8) & (r < 10) & (s < 9) {
                Some(axial_to_chess_coords(q + 3, r + 1))
            } else {
                None
            },
            if (q < 8) & (r < 9) & (s < 10) {
                Some(axial_to_chess_coords(q + 3, r + 2))
            } else {
                None
            },
            if (q < 9) & (r < 8) & (s > 0) {
                Some(axial_to_chess_coords(q + 2, r + 3))
            } else {
                None
            },
            //top
            if (q < 10) & (r < 8) & (s > 1) {
                Some(axial_to_chess_coords(q + 1, r + 3))
            } else {
                None
            },
            if (q > 0) & (r < 9) & (s > 2) {
                Some(axial_to_chess_coords(q - 1, calc_r(q - 1, s - 3)))
            } else {
                None
            },
            //bottom
            if (q < 10) & (r > 1) & (s < 8) {
                Some(axial_to_chess_coords(q + 1, r - 2))
            } else {
                None
            },
            if (q > 0) & (r > 2) & (s < 9) {
                Some(axial_to_chess_coords(q - 1, r - 3))
            } else {
                None
            },
        ];

        let move_list: Vec<Hexagon> = move_options.into_iter().filter_map(|x| x).collect();

        KnightMoves {
            move_list: move_list,
        }
    }

    pub fn drop_arm(&mut self) {
        // drop the current arm of valid moves.
        // In this case, that is just removing a single move.
        // e.g. if a piece is blocking the remainder of the arm
        self.move_list.pop();
    }
}

impl Iterator for KnightMoves {
    type Item = Hexagon;
    fn next(&mut self) -> Option<Hexagon> {
        if let Some(val) = self.move_list.pop() {
            Some(val)
        } else {
            None
        }
    }
}

pub fn pawn_moves_double_jump(hexagon: &Hexagon, color: &Color, board: &Board) -> Option<Hexagon> {
    let mut jump = None;
    let (q, r) = chess_to_axial_coords(&hexagon);
    let s = calc_s(q, r);
    // if pawn is on starting rank, it can double jump
    // ... but only if it's ordinary square isn't blocked
    if matches!(color, Color::White) & (((s == 6) & (q < 6)) | ((r == 4) & (q > 5))) {
        if board
            .occupied_squares
            .get(&axial_to_chess_coords(q, r + 1))
            .is_none()
        {
            jump = Some(axial_to_chess_coords(q, r + 2));
        };
    } else if matches!(color, Color::Black) & (((s == 4) & (q > 4)) | (r == 6) & (q < 6)) {
        if board
            .occupied_squares
            .get(&axial_to_chess_coords(q, r - 1))
            .is_none()
        {
            jump = Some(axial_to_chess_coords(q, r - 2));
        }
    }
    jump
}

pub fn pawn_moves_not_attacking(hexagon: &Hexagon, color: &Color) -> Vec<Hexagon> {
    let mut moves = Vec::<Hexagon>::new();
    let (q, r) = chess_to_axial_coords(&hexagon);

    // add the normal, single forward move
    moves.push(axial_to_chess_coords(
        q,
        if matches!(color, Color::White) {
            r + 1
        } else {
            r - 1
        },
    ));

    // ignore en passant for now, it's going to require a decently big restructure

    // ignore promotion for now, again a future headache

    moves
}

pub fn pawn_moves_attacking(hexagon: &Hexagon, color: &Color) -> Vec<Hexagon> {
    let mut valid_moves = Vec::<Hexagon>::new();
    let (q, r) = chess_to_axial_coords(&hexagon);
    let s = calc_s(q, r);
    // white attacking
    if matches!(color, Color::White) & (s > 1) {
        if q > 0 {
            valid_moves.push(axial_to_chess_coords(q - 1, r));
        }
        if q < 10 {
            valid_moves.push(axial_to_chess_coords(q + 1, r + 1))
        }
    } else if r > 0 {
        if q > 0 {
            valid_moves.push(axial_to_chess_coords(q - 1, r - 1));
        }
        if q < 10 {
            valid_moves.push(axial_to_chess_coords(q + 1, r))
        }
    }
    valid_moves
}

pub fn pawn_moves(
    hexagon: &Hexagon,
    color: &Color,
    board: &Board,
) -> (Vec<Hexagon>, Option<Hexagon>) {
    let mut valid_moves = Vec::<Hexagon>::new();

    let attacking = pawn_moves_attacking(hexagon, color);
    let not_attacking = pawn_moves_not_attacking(hexagon, color);
    let double_jump = pawn_moves_double_jump(hexagon, color, board);

    for hex in attacking {
        if board.en_passant.is_some() {
            let virtual_pawn = convert_en_passant_to_virtual_pawn(&hex, *color);
            if virtual_pawn == board.en_passant.unwrap() {
                valid_moves.push(hex);
            }
        }
        if let Some(occupied_hex) = board.occupied_squares.get(&hex) {
            if &occupied_hex.color != color {
                valid_moves.push(hex);
            };
        }
    }

    for hex in not_attacking {
        if let Some(_) = board.occupied_squares.get(&hex) {
        } else {
            valid_moves.push(hex);
        }
    }

    if double_jump.is_some() {
        if let Some(_) = board.occupied_squares.get(&double_jump.unwrap()) {
        } else {
            valid_moves.push(double_jump.unwrap());
        }
    }

    (valid_moves, double_jump)
}
