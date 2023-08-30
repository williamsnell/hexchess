import { readable } from "svelte/store";
import { Color, PieceType } from "./hexchess_logic";


export function get_piece_asset(color: string, piece: string) {
    return `/src/assets/pieces/${piece.toLowerCase()}_${color.toLowerCase()}.svg`;
  }