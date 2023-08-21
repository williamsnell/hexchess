const vertical_hexagons_per_column = [6, 7, 8, 9, 10, 11, 10, 9, 8, 7, 6];
const files = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "k", "l"];

export const char_to_file = {
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
  "A": 0,
  "B": 1,
  "C": 2,
  "D": 3,
  "E": 4,
  "F": 5,
  "G": 6,
  "H": 7,
  "I": 8,
  "K": 9,
  "L": 10
};

function deg2rad(x) {
  return x * Math.PI / 180;
}

export function get_polygon_points(size, x, y, number_of_sides) {
  var points = [];
  for (var i = 1; i <= number_of_sides; i += 1) {
    points.push({"x": x + size * Math.cos(i * 2 * Math.PI / number_of_sides), "y": y + size * Math.sin(i * 2 * Math.PI / number_of_sides)});
  }
  return points;
}

function draw_polygon(size, x, y, number_of_sides, colour = "#000000", context) {
  context.beginPath();
  // context.moveTo(x + size, y);

  var points = get_polygon_points(size, x, y, number_of_sides);

  points.forEach((val) => context.lineTo(val.x, val.y));

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

function calc_column_x_position(column, hex_size, canvas, orientation) {
  var x_start = canvas.width / 2 - get_centre_of_hex_structure(11) * calc_hex_x_offset(hex_size);
  // Calculate the x-offset of a given column
  return (1 - orientation) * canvas.width/2 + orientation * (x_start + column * calc_hex_x_offset(hex_size));
}

export function calc_column_y_positions(number_of_hexagons, y_center, size, canvas, orientation) {
  var y2 = get_centre_of_hex_structure(number_of_hexagons) * calc_hex_ysize(size) + y_center;
  var y = [];
  for (var i = 0; i < number_of_hexagons; i += 1) {
    y.push((1 - orientation) * canvas.height/2 + orientation * (size * i * (2 * Math.sin(deg2rad(120))) + y2));
  }
  return y;
}


function calc_row_position(row, column, canvas, hex_size, orientation) {
  var number_of_vertical_hexes = vertical_hexagons_per_column[column];
  var min_y = canvas.height / 2 - calc_hex_ysize(hex_size) * (number_of_vertical_hexes - 1) / 2;
  return (1 - orientation) * canvas.height/2 + orientation * (min_y + calc_hex_ysize(hex_size) * row);
}

function get_hexagon_position(rank, file, canvas, hex_size, orientation) {
  var x = calc_column_x_position(rank, hex_size, canvas, orientation);
  var y = calc_row_position(file, rank, canvas, hex_size, orientation);
  return [x, y];
}

function isInsidePolygon(polygon, mouseX, mouseY) {
  var c = false;

  for (let i = 1, j = 0; i < polygon.length; i++, j++) {
    const ix = polygon[i].x;
    const iy = polygon[i].y;
    const jx = polygon[j].x;
    const jy = polygon[j].y;
    const iySide = (iy > mouseY);
    const jySide = (jy > mouseY);

    if (iySide != jySide) {
      const intersectX = (jx - ix) * (mouseY - iy) / (jy - iy) + ix;
      if (mouseX < intersectX)
        c = !c;
    }
  }
  return c;
}

export {
  deg2rad, draw_hexagon, calc_hex_xsize, calc_hex_ysize, calc_hex_x_offset,
  get_centre_of_hex_structure, calc_column_x_position, get_hexagon_position, vertical_hexagons_per_column, files, isInsidePolygon
};