import { get_piece_asset } from "./assets.js";
import { get_hexagon_position } from "./get_hexagon_position.js";
import { char_to_file, type Hexagon, type Piece, type Rank, Color, HexagonPattern } from "./hexchess_logic";
import { writable } from "svelte/store";

const board_json = { "op": "BoardState", "board": { "occupied_squares": { "k7": { "piece_type": "Pawn", "color": "Black" }, "g1": { "piece_type": "King", "color": "White" }, "f3": { "piece_type": "Bishop", "color": "White" }, "g4": { "piece_type": "Pawn", "color": "White" }, "b7": { "piece_type": "Pawn", "color": "Black" }, "e7": { "piece_type": "Pawn", "color": "Black" }, "h3": { "piece_type": "Pawn", "color": "White" }, "d9": { "piece_type": "Knight", "color": "Black" }, "f5": { "piece_type": "Pawn", "color": "White" }, "i1": { "piece_type": "Rook", "color": "White" }, "f1": { "piece_type": "Bishop", "color": "White" }, "h1": { "piece_type": "Knight", "color": "White" }, "e10": { "piece_type": "Queen", "color": "Black" }, "g7": { "piece_type": "Pawn", "color": "Black" }, "k1": { "piece_type": "Pawn", "color": "White" }, "i7": { "piece_type": "Pawn", "color": "Black" }, "f10": { "piece_type": "Bishop", "color": "Black" }, "e1": { "piece_type": "Queen", "color": "White" }, "c7": { "piece_type": "Pawn", "color": "Black" }, "d3": { "piece_type": "Pawn", "color": "White" }, "d7": { "piece_type": "Pawn", "color": "Black" }, "c2": { "piece_type": "Pawn", "color": "White" }, "d1": { "piece_type": "Knight", "color": "White" }, "h7": { "piece_type": "Pawn", "color": "Black" }, "f9": { "piece_type": "Bishop", "color": "Black" }, "b1": { "piece_type": "Pawn", "color": "White" }, "c1": { "piece_type": "Rook", "color": "White" }, "f2": { "piece_type": "Bishop", "color": "White" }, "g10": { "piece_type": "King", "color": "Black" }, "c8": { "piece_type": "Rook", "color": "Black" }, "h9": { "piece_type": "Knight", "color": "Black" }, "f11": { "piece_type": "Bishop", "color": "Black" }, "e4": { "piece_type": "Pawn", "color": "White" }, "i2": { "piece_type": "Pawn", "color": "White" }, "f7": { "piece_type": "Pawn", "color": "Black" }, "i8": { "piece_type": "Rook", "color": "Black" } }, "en_passant": null, "current_player": "White" } }

class Board {
    occupied_squares!: Record<Hexagon, Piece>;
    en_passant!: Hexagon | null // although this is property may be null,
    // it is always required in the message
    current_player!: Color
}


export function parse_hexagon_string(position: Hexagon) {
    const [, rank_string, file_string] = position.match(HexagonPattern);
    const rank = char_to_file[rank_string];
    const file = Number(file_string) - 1;
    return { rank, file };
}

function instantiate_pieces(board: Board) {
    const active_pieces = [];
    for (const [hex, piece] of Object.entries(board.occupied_squares)) {
        const position = get_hexagon_position(hex);
        const img_source = get_piece_asset(piece.color, piece.piece_type);
        active_pieces.push({ "hex": hex, "position": { "x": position[0], "y": position[1] }, "img_src": img_source, "alt": `${piece.color} ${piece.piece_type}` })
    }
    return active_pieces;
}

export const board = writable(instantiate_pieces(board_json.board), () => { return () => { } });

