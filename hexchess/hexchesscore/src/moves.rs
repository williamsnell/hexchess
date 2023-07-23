use std::iter::zip;

use crate::hexchesscore::Hexagon;

pub fn get_rank_length(rank: u8) -> Option<u8> {
    match rank {
        1 => Some(6),
        2 => Some(7),
        3 => Some(8),
        4 => Some(9),
        5 => Some(10),
        6 => Some(11),
        7 => Some(10),
        8 => Some(9),
        9 => Some(8),
        10 => Some(7),
        11 => Some(6),
        _ => None,
    }
}

pub struct RookMoves {
    move_list: Vec<Vec<Hexagon>>,
}

impl RookMoves {
    pub fn new(position: Hexagon) -> RookMoves {
        /// Get the moves a rook could make, from the current position, assuming
        /// no other pieces. These moves spiral out from the rook, so closer hexagons
        /// are returned earlier in the result
        // loop through arms
        let arm_top: Vec<Hexagon> = (position.file + 1
            ..=get_rank_length(position.rank).unwrap())
            .rev()
            .map(|x| Hexagon {
                rank: position.rank,
                file: x,
            })
            .collect();

        let arm_top_left: Vec<Hexagon> = (1..position.rank)
            .map(|x| Hexagon {
                rank: x,
                file: position.file,
            })
            .collect();

        let arm_bottom_left: Vec<Hexagon> = zip((1..position.rank), (1..position.file))
            .map(|(x, y)| Hexagon { rank: x, file: y })
            .collect();

        let arm_bottom: Vec<Hexagon> = (1..position.file)
            .map(|x| Hexagon {
                rank: position.rank,
                file: x,
            })
            .collect();

        let arm_bottom_right: Vec<Hexagon> =
            zip((position.rank + 1..=11).rev(), (1..position.file))
                .map(|(x, y)| Hexagon { rank: x, file: y })
                .collect();

        let arm_top_right: Vec<Hexagon> = (position.rank + 1..=11)
            .rev()
            .map(|x| Hexagon {
                rank: x,
                file: position.file,
            })
            .collect();

        RookMoves {
            move_list: vec![
                arm_top,
                arm_top_left,
                arm_bottom_left,
                arm_bottom,
                arm_bottom_right,
                arm_top_right,
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
