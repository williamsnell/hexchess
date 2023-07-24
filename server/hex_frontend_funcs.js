const vertical_hexagons_per_column = [6, 7, 8, 9, 10, 11, 10, 9, 8, 7, 6];

function deg2rad(x) {
  return x * Math.PI / 180;
}

function draw_polygon(size, x, y, number_of_sides, colour = "#000000", context) {
  context.beginPath();
  context.moveTo(x + size, y);

  for (var i = 1; i <= number_of_sides; i += 1) {
    context.lineTo(x + size * Math.cos(i * 2 * Math.PI / number_of_sides), y + size * Math.sin(i * 2 * Math.PI / number_of_sides));
  }

  context.strokeStyle = "#e0c6a1";
  context.fillStyle = colour;
  context.lineWidth = 1;
  context.stroke();
  context.fill();
}

function draw_hexagon(size, x, y, colour, ctx) {
  draw_polygon(size, x, y, 6, colour, ctx);
}

function calc_hex_xsize(hex_size) {
  return hex_size * 2;
}

function calc_hex_x_offset(hex_size) {
  return hex_size * 1.5;
}

function calc_hex_ysize(hex_size) {
  return hex_size * 2 * (-Math.sin(deg2rad(120)));
}

function get_centre_of_hex_structure(number_of_hexagons) {
  return (number_of_hexagons / 2 - 0.5);
}

function calc_column_position(column, hex_size, canvas) {
  var x_start = canvas.width / 2 - get_centre_of_hex_structure(11) * calc_hex_x_offset(hex_size);
  // Calculate the x-offset of a given column
  return x_start + column * calc_hex_x_offset(hex_size);
}

function calc_row_position(row, column, canvas, hex_size) {
  var number_of_vertical_hexes = vertical_hexagons_per_column[column];
  var min_y = canvas.height / 2 - calc_hex_ysize(hex_size) * (number_of_vertical_hexes - 1) / 2;
  return min_y + calc_hex_ysize(hex_size) * row;
}

function get_hexagon_position(rank, file, canvas, hex_size) {
  var x = calc_column_position(rank - 1, hex_size, canvas);
  var y = calc_row_position(file - 1, rank - 1, canvas, hex_size);
  return [x, y];
}

export { deg2rad, draw_hexagon, calc_hex_xsize, calc_hex_ysize, calc_hex_x_offset, 
  get_centre_of_hex_structure, calc_column_position, get_hexagon_position, vertical_hexagons_per_column};