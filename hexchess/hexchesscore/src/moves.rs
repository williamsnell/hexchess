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

pub fn axial_to_chess_coords(q: u8, r: u8) -> Hexagon {
    println!("=> q: {:?}, r: {:?}", q, r);

    let rank = q;
    let file = r + get_rank_length(q).unwrap() + (if rank < 6 { 0 } else { rank - 5 }) - rank - 6;
    println!("=> rank: {:?}, file: {:?}", rank, file);
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
        println!("q: {:?}, r: {:?}", q, r);
        let arm_top: Vec<Hexagon> = (position.file + 1..get_rank_length(position.rank).unwrap())
            .rev()
            .map(|x| Hexagon {
                rank: position.rank,
                file: x,
            })
            .collect();

        let arm_top_left: Vec<Hexagon> = (0..q).map(|x| axial_to_chess_coords(x, r)).collect();

        // let arm_bottom_left: Vec<Hexagon> = (0..q)
        //     .map(|x| axial_to_chess_coords(x, x + r - q))
        //     .collect();

        // let arm_bottom: Vec<Hexagon> = (0..position.file)
        //     .map(|x| Hexagon {
        //         rank: position.rank,
        //         file: x,
        //     })
        //     .collect();

        // let arm_bottom_right: Vec<Hexagon> = (q..=get_rank_length(q - r).unwrap())
        //     .rev()
        //     .map(|x| axial_to_chess_coords(x, r))
        //     .collect();

        // let arm_top_right: Vec<Hexagon> = (q + 1..get_rank_length(q - 1).unwrap())
        //     .rev()
        //     .map(|x| axial_to_chess_coords(x, x + r - q)).collect();

        RookMoves {
            move_list: vec![
                arm_top,
                arm_top_left,
                // arm_bottom_left,
                // arm_bottom,
                // arm_bottom_right,
                // arm_top_right,
            ],
        }
    }
    pub fn drop_arm(&mut self) {
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

/* "a1", "b2", "c3", "d4", "e5",
"e6", "d6", "c6", "b6", "a6",
"f1", "f2", "f3", "f4", "f5",
"f7", "f8", "f9", "f10", "f11",
"g5", "h4", "i3", "k2", "l1",
"g6", "h6", "i6", "k6", "l6",*/
