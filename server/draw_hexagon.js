import { deg2rad, draw_hexagon, calc_hex_ysize, get_centre_of_hex_structure, 
  calc_column_position, vertical_hexagons_per_column, get_hexagon_position, files } from "./hex_frontend_funcs.js";

var canvas = document.getElementById("hexagon");
var ctx = canvas.getContext("2d");

function draw_hexagon_column(number_of_hexagons, x, y_center, size) {
  var y2 = get_centre_of_hex_structure(number_of_hexagons) * calc_hex_ysize(size) + y_center;

  var colour;

  for (var i = 0; i < number_of_hexagons; i += 1) {
    if ((i + number_of_hexagons) % 3 == 0) {
      colour = "#ce946d";
    } else if ((i + number_of_hexagons) % 3 == 1) {
      colour = "#cec46d";
    } else if ((i + number_of_hexagons) % 3 == 2) {
      colour = "#ce6d77";
    }
    draw_hexagon(size, x, size * i * (2 * Math.sin(deg2rad(120))) + y2, colour, ctx);
  }
}

var hex_size = canvas.width * 0.05;

function draw_hex_callback(element, index) {
  draw_hexagon_column(element, calc_column_position(index, hex_size, canvas), canvas.height * 0.5, hex_size);
}

function label_hexes(context, canvas, hex_size) {
  for (var i = 0; i < files.length; i++) {
    for (var j = 0; j <= vertical_hexagons_per_column[i] - 1; j++) {
      var x, y;
      [x, y] = get_hexagon_position(i, j, canvas, hex_size);
      context.fillStyle = "#000000";
      context.font = "15px arial";
      context.fillText((String(files[i]).toUpperCase() + String(j + 1)), x, y);
      // var q = i;
      var q = i;
      var r = j + i - (vertical_hexagons_per_column[i] - 6) + (i < 6 ? 0 : 5 - i);
      var s = q - r + 5;
      context.fillText(q, x - hex_size/4, y - hex_size / 2);
      context.fillStyle = "#F0F0F0";
      context.fillText(r, x + hex_size / 3, y + hex_size * 0.5);
      context.fillStyle = "#F0F000";
      context.fillText(s, x - hex_size * 0.9, y + hex_size /3);
    }
  }
}

function draw_board() {
  vertical_hexagons_per_column.forEach(draw_hex_callback);
}

draw_board();

function draw_dot(rank, file) {
  var x, y;
  [x, y] = get_hexagon_position(rank, file, canvas, hex_size);
  
  ctx.beginPath();
  ctx.fillStyle = "#000000";
  ctx.lineStyle = "#000000";
  ctx.lineWidth = 0;
  ctx.arc(x, y, hex_size * 0.3, 0, 2 * Math.PI);
  ctx.fill();
  ctx.stroke();
}

function parse_moves(text) {
  var payload = JSON.parse(text);
  console.log(payload);
  var moves = payload["moves"];
  moves.forEach((val) => draw_dot(val["rank"], val["file"]));
  return text;
}

// Draw all positions from a file
fetch("moves.json").then(res => res.text()).then(text => parse_moves(text)).catch(e => console.error(e));

label_hexes(ctx, canvas, hex_size);


