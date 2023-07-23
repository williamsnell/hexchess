import("./hex_frontend_funcs.js");

var canvas = document.getElementById('hexagon')
var ctx = canvas.getContext('2d');

function deg2rad(x) {
  return x * Math.PI / 180;
}

function draw_polygon(size, x, y, number_of_sides, colour = "#000000") {
  ctx.beginPath();
  ctx.moveTo(x + size, y);

  for (var i = 1; i <= number_of_sides; i += 1) {
    ctx.lineTo(x + size * Math.cos(i * 2 * Math.PI / number_of_sides), y + size * Math.sin(i * 2 * Math.PI / number_of_sides));
  }

  ctx.strokeStyle = "#e0c6a1";
  ctx.fillStyle = colour;
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.fill();
}

function draw_hexagon(size, x, y, colour) {
  draw_polygon(size, x, y, 6, colour);
}

function calc_hex_xsize(hex_size) {
  return hex_size * 2;
}

function calc_hex_x_offset(hex_size) {
  return hex_size * 1.5
}

function calc_hex_ysize(hex_size) {
  return hex_size * 2 * (-Math.sin(deg2rad(120)));
}

function get_centre_of_hex_structure(number_of_hexagons) {
  return (number_of_hexagons / 2 - 0.5);
}

function calc_column_position(column, hex_size) {
  var x_start = canvas.width / 2 - get_centre_of_hex_structure(11) * calc_hex_x_offset(hex_size);
  // Calculate the x-offset of a given column
  return x_start + column * calc_hex_x_offset(hex_size);
}

function draw_hexagon_column(number_of_hexagons, x, y_center, size) {
  y2 = get_centre_of_hex_structure(number_of_hexagons) * calc_hex_ysize(size)  + y_center;
  
  var colour;

  for (var i = 0; i < number_of_hexagons; i += 1) {
    if ((i + number_of_hexagons) % 3 == 0) {
      colour = "#ce946d";
    } else if ((i + number_of_hexagons) % 3 == 1) {
      colour = "#cec46d";
    } else if ((i + number_of_hexagons) % 3 == 2) {
      colour = "#ce6d77";
    }
    draw_hexagon(size = size, x = x, y = size * i * (2 * Math.sin(deg2rad(120))) + y2, colour);
  }
}

const cells = [6, 7, 8, 9, 10, 11, 10, 9, 8, 7, 6]

function draw_hex_callback(element, index) {
  var hex_size = canvas.width * 0.05;

  draw_hexagon_column(element, calc_column_position(index, hex_size), canvas.height * 0.5, hex_size);
}

function draw_board() {
  cells.forEach(draw_hex_callback);
}

draw_board();

window.onresize = draw_board;