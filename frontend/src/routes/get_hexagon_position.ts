import { type Hexagon, vertical_hexagons_per_column } from "./hexchess_logic";
import { parse_hexagon_string } from "./board_state.js";

function calc_column_x_position(column) {
    // Calculate the x-offset of a given column
    return column / 11;
}
function calc_row_position(row, column) {
    const number_of_vertical_hexes = vertical_hexagons_per_column[column];
    const min_y = (number_of_vertical_hexes - 1) / 2;
    return ((-min_y + row + 12)) / 11;
}
export function get_hexagon_position(position: Hexagon) {
    const { rank, file } = parse_hexagon_string(position);
    return [calc_column_x_position(rank), calc_row_position(file, rank)];
}
