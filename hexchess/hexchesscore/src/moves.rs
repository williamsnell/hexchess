use std::{
    cmp::{max, min},
    iter::zip,
};

use crate::hexchesscore::Hexagon;

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

pub struct RookMoves {
    move_list: Vec<Vec<Hexagon>>,
}

impl RookMoves {
    pub fn new(position: Hexagon) -> RookMoves {
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

        RookMoves {
            move_list: vec![
                arm_top,
                arm_bottom,
                arm_top_left,
                arm_bottom_right,
                arm_bottom_left,
                arm_top_right,
            ],
        }
    }
    pub fn drop_arm(&mut self) {
        // drop the current arm of valid rook moves
        // e.g. if a piece is blocking the remainder of the arm
        self.move_list.pop();
    }
}

impl Iterator for RookMoves {
    type Item = Hexagon;
    fn next(&mut self) -> Option<Hexagon> {
        // if let Some(output) = (*self.move_list.last().unwrap()).pop() {
        //     Some(output)
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

pub struct BishopMoves {
    move_list: Vec<Vec<Hexagon>>,
}

impl BishopMoves {
    pub fn new(position: Hexagon) -> BishopMoves {
        // Get the moves a bishop could make, from the current position, assuming
        // no other pieces. These moves spiral out from the bishop, so closer hexagons
        // are returned earlier in the result
        let (q, r) = chess_to_axial_coords(&position);
        let s = calc_s(q, r);

        let arm_left: Vec<Hexagon> =
            zip((q % 2..q).step_by(2).rev(), zip((0..r).rev(), (0..s).rev()))
                .map(|(x, (y, z))| axial_to_chess_coords(x, y))
                .collect();

        let arm_right: Vec<Hexagon> =
            zip((q+2..=10).step_by(2), zip((r+1..=10), (s+1..=10)))
                .map(|(x, (y, z))| axial_to_chess_coords(x, y))
                .collect();

        let arm_down_left: Vec<Hexagon> =
            zip((r % 2..r).step_by(2).rev(), zip((0..q).rev(), (s+1..=10)))
                .map(|(x, (y, z))| axial_to_chess_coords(y, x))
                .collect();

        let arm_up_right: Vec<Hexagon> =
            zip((r+2..=10).step_by(2), zip((q+1..=10), (0..s).rev()))
                .map(|(x, (y, z))| axial_to_chess_coords(y, x))
                .collect();

        let arm_up_left: Vec<Hexagon> = zip((s % 2..s).step_by(2).rev(), zip((0..q).rev(), (r+1..=10)))
        .map(|(x, (y, z))| axial_to_chess_coords(y, z))
        .collect();

        let arm_down_right: Vec<Hexagon> =
            zip((s+2..=10).step_by(2), zip((q+1..=10), (0..r).rev()))
                .map(|(x, (y, z))| axial_to_chess_coords(y, z))
                .collect();
        
        BishopMoves {
            move_list: vec![
                arm_left,
                arm_right,
                arm_down_left,
                arm_up_right,
                arm_up_left,
                arm_down_right,
                ],
        }
    }
    pub fn drop_arm(&mut self) {
        // drop the current arm of valid rook moves
        // e.g. if a piece is blocking the remainder of the arm
        self.move_list.pop();
    }
}

impl Iterator for BishopMoves {
    type Item = Hexagon;
    fn next(&mut self) -> Option<Hexagon> {
        // if let Some(output) = (*self.move_list.last().unwrap()).pop() {
        //     Some(output)
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
