export type Rank = 'a'|'b'|'c'|'d'|'e'|'f'|'g'|'h'|'i'|'k'|'l'

export type HexFile = '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'10'

export const vertical_hexagons_per_column = [6, 7, 8, 9, 10, 11, 10, 9, 8, 7, 6];

// export type Hexagon = `${Rank}${number}`;
export const HexagonPattern = /^([a-z])(\d+)$/;
export type Hexagon = string;

export enum PieceType {
    Pawn = "Pawn",
    Rook = "Rook",
    Knight = "Knight",
    Bishop = "Bishop",
    Queen = "Queen",
    King = "King",
};

export enum Color {
    Black = "Black",
    White = "White"
}

export class Piece {
    piece_type!: PieceType
    color!: Color
}


export const char_to_file: Record<Rank, number> = {
    "a": 0,
    "b": 1,
    "c": 2,
    "d": 3,
    "e": 4,
    "f": 5,
    "g": 6,
    "h": 7,
    "i": 8,
    "k": 9,
    "l": 10,
  };