import { type Hexagon, vertical_hexagons_per_column } from "./hexchess_logic";
import { parse_hexagon_string } from "./board_state.js";

function calc_column_x_position(column) {
    // Calculate the x-offset of a given column

    // there are 23 edges (11 horizontal, 12 diagonal) across the span
    // of the board
    // that equates to (11 w units + 6 w units)
    return (column - 5) / 11.32;
}
function calc_row_position(row, column) {
    const number_of_vertical_hexes = vertical_hexagons_per_column[column];
    const min_y = (number_of_vertical_hexes - 1) / 2;
    return ((-min_y + row)) / 11;
}
export function get_hexagon_position(position: Hexagon) {
    const { rank, file } = parse_hexagon_string(position);
    return [calc_column_x_position(rank), calc_row_position(file, rank)];
}
