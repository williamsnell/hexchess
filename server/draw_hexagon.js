import {
  draw_hexagon,
  calc_column_x_position, calc_column_y_positions, vertical_hexagons_per_column, get_hexagon_position, files, isInsidePolygon, get_polygon_points
} from "./hex_frontend_funcs.js";

var canvas = document.getElementById("hexagon");
var ctx = canvas.getContext("2d");

function get_hex_color(i, number_of_hexagons) {
  var colour;
  if ((i + number_of_hexagons) % 3 == 0) {
    colour = "#ce946d";
  } else if ((i + number_of_hexagons) % 3 == 1) {
    colour = "#cec46d";
  } else if ((i + number_of_hexagons) % 3 == 2) {
    colour = "#ce6d77";
  }
  return colour;
}

function draw_hexagon_column(number_of_hexagons, x, y_center, size) {
  var y_positions = calc_column_y_positions(number_of_hexagons, y_center, size);
  y_positions.forEach((val, index) => draw_hexagon(size, x, val, get_hex_color(index, number_of_hexagons), ctx));
  return y_positions;
}

var hex_size = canvas.width * 0.05;

function draw_hex_callback(element, index) {
  var x_positions = Array(element).fill([calc_column_x_position(index, hex_size, canvas)]).flat();
  var y_positions = draw_hexagon_column(element, x_positions[0], canvas.height * 0.5, hex_size);
  var out = [];
  for (let i = 0; i < x_positions.length; i++) {
    out.push({ "x": x_positions[i], "y": y_positions[i] });
  }
  return out;
}

function label_hexes(context, canvas, hex_size) {
  var hexes = {};
  for (var i = 0; i < files.length; i++) {
    for (var j = 0; j <= vertical_hexagons_per_column[i] - 1; j++) {
      var x, y;
      [x, y] = get_hexagon_position(i, j, canvas, hex_size);
      context.fillStyle = "#000000";
      context.font = "15px arial";
      var chess_coord = (String(files[i]).toUpperCase() + String(j + 1));
      context.fillText(chess_coord, x, y);

      hexes[`${x},${y}`] = chess_coord;

      // cubic coordinates
      var q = i;
      var r = j + i - (vertical_hexagons_per_column[i] - 6) + (i < 6 ? 0 : 5 - i);
      var s = q - r + 5;
      context.fillText(q, x - hex_size / 4, y - hex_size / 2);
      context.fillStyle = "#F0F0F0";
      context.fillText(r, x + hex_size / 3, y + hex_size * 0.5);
      context.fillStyle = "#F0F000";
      context.fillText(s, x - hex_size * 0.9, y + hex_size / 3);
    }
  }
  return hexes;
}

function draw_board() {
  var positions = [];
  for (let i = 0; i < vertical_hexagons_per_column.length; i++) {
    positions.push(draw_hex_callback(vertical_hexagons_per_column[i], i));
  }
  return positions.flat();
}

const hex_positions = draw_board();

function draw_dot_x_y(x, y, radius = 0.3 * hex_size) {
  ctx.beginPath();
  ctx.fillStyle = "#000000";
  ctx.lineStyle = "#000000";
  ctx.lineWidth = 0;
  ctx.arc(x, y, radius, 0, 2 * Math.PI);
  ctx.fill();
  ctx.stroke();
  ctx.closePath();
}

function draw_dot(rank, file) {
  var x, y;
  [x, y] = get_hexagon_position(rank, file, canvas, hex_size);
  draw_dot_x_y(x, y);
}

function parse_moves(text) {
  var payload = JSON.parse(text);
  var moves = payload["moves"];
  moves.forEach((val) => draw_dot(val["rank"], val["file"]));
  return text;
}

// Draw all positions from a file
fetch("moves.json").then(res => res.text()).then(text => parse_moves(text)).catch(e => console.error(e));

var hex_labels = label_hexes(ctx, canvas, hex_size);



function setup_websocket() {
  const BACKEND_URL = "ws://" + window.location.hostname + ":8080";
  const socket = new WebSocket(BACKEND_URL);
  socket.onmessage = (msg) => parse_moves(msg.data);
  socket.onerror = (err) => console.error(err);
  socket.onclose = () => console.log("Socket Closed");

  return socket;
}

var socket = setup_websocket();

function select_hexagon(event) {
  const mouse_x = event.offsetX;
  const mouse_y = event.offsetY;

  draw_board();
  label_hexes(ctx, canvas, hex_size);


  for (let i = 0; i < hex_positions.length; i++) {
    if (Math.sqrt((mouse_x - hex_positions[i].x) ** 2 + (mouse_y - hex_positions[i].y) ** 2) < hex_size * 0.866) {
      ctx.lineWidth = 0;
      var hex_x = hex_positions[i].x;
      var hex_y = hex_positions[i].y;

      if (socket.readyState == socket.OPEN) {
        socket.send(hex_labels[`${hex_x},${hex_y}`]);
      }
      else {
        socket = setup_websocket(); 
      }
      break;
    }
  }

}

canvas.addEventListener("click", select_hexagon);

