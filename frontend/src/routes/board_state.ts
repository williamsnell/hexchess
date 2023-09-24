import { get_piece_asset } from "./assets.js";
import { get_hexagon_position } from "./get_hexagon_position.js";
import { char_to_file, type Hexagon, type Piece, type Rank, Color, HexagonPattern } from "./hexchess_logic";
import { writable } from "svelte/store";

const board_json = { "op": "BoardState", "board": { "occupied_squares": { }, "en_passant": null, "current_player": "White" } }

export class Board {
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

export function instantiate_pieces(board: Board) {
    const active_pieces = [];
    for (const [hex, piece] of Object.entries(board.occupied_squares)) {
        const position = get_hexagon_position(hex);
        const img_source = get_piece_asset(piece.color, piece.piece_type);
        active_pieces.push({ "hex": hex, "position": { "x": position[0], "y": position[1] }, "img_src": img_source, "alt": `${piece.color} ${piece.piece_type}` })
    }
    return active_pieces;
}

export const board = writable(instantiate_pieces(board_json.board));

export function show_available_moves(hexagon, user_id, socket_send) {
    // send a message to the websocket to get the 
    // valid moves.
    socket_send(
        `{"op": "GetMoves",
          "user_id": "${user_id}",
          "hexagon": "${hexagon}"}`
      );
    }

export function move_piece(start_hex, destination_hex, user_id, socket_send) {
        // send a message to register a piece moving
        socket_send(
          `{"op": "RegisterMove",
              "user_id": "${user_id}",
              "start_hexagon": "${start_hex}",
              "final_hexagon": "${destination_hex}",
              "promotion_choice": "Queen"}`
        );
      }