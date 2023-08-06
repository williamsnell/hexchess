import {
  draw_hexagon,
  calc_column_x_position, calc_column_y_positions, vertical_hexagons_per_column, get_hexagon_position, files
} from "./hex_frontend_funcs.js";

const draw_labels = false;

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

var knight = new Image();
knight.src = "assets/pieces/knight_white.svg";

function draw_hex_callback(element, index) {
  var x_positions = Array(element).fill([calc_column_x_position(index, hex_size, canvas)]).flat();
  var y_positions = draw_hexagon_column(element, x_positions[0], canvas.height * 0.5, hex_size);
  var out = [];
  for (let i = 0; i < x_positions.length; i++) {
    out.push({ "x": x_positions[i], "y": y_positions[i] });
  }
  return out;
}

function label_hexes(context, canvas, hex_size, show = true) {
  var hexes = {};
  for (var i = 0; i < files.length; i++) {
    for (var j = 0; j <= vertical_hexagons_per_column[i] - 1; j++) {
      var x, y;
      [x, y] = get_hexagon_position(i, j, canvas, hex_size);
      context.fillStyle = "#000000";
      context.font = "15px arial";
      var chess_coord = (String(files[i]).toUpperCase() + String(j + 1));

      hexes[`${x},${y}`] = chess_coord;

      // cubic coordinates
      var q = i;
      var r = j + i - (vertical_hexagons_per_column[i] - 6) + (i < 6 ? 0 : 5 - i);
      var s = q - r + 5;
      if (show) {
        context.fillText(chess_coord, x, y);
        context.fillText(q, x - hex_size / 4, y - hex_size / 2);
        context.fillStyle = "#F0F0F0";
        context.fillText(r, x + hex_size / 3, y + hex_size * 0.5);
        context.fillStyle = "#F0F000";
        context.fillText(s, x - hex_size * 0.9, y + hex_size / 3);
      }
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

draw_board();

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

var valid_moves = [];

function parse_moves(text) {
  var payload = JSON.parse(text);
  valid_moves = payload["moves"];
  valid_moves.forEach((val) => draw_dot(val["rank"], val["file"]));
  return text;
}

var hex_labels = label_hexes(ctx, canvas, hex_size, draw_labels);



function setup_websocket() {
  const BACKEND_URL = "ws://" + window.location.hostname + ":7979";
  const socket = new WebSocket(BACKEND_URL);
  socket.onmessage = (msg) => parse_moves(msg.data);
  socket.onerror = (err) => console.error(err);
  socket.onclose = () => console.log("Socket Closed");

  return socket;
}

var socket = setup_websocket();


// request the board position, and display it
const Pieces = {
  Pawn: "Pawn",
  Rook: "Rook",
  Knight: "Knight",
  Bishop: "Bishop",
  Queen: "Queen",
  King: "King",
};

const Color = {
  White: "White",
  Black: "Black",
}

function get_piece_asset(color, piece) {
  return `assets/pieces/${piece.toLowerCase()}_${color.toLowerCase()}.svg`;
}

var board;

fetch("starting_moves.json").then((res) => res.json()).then(res => board = res).then((res) => display_board(res));

async function display_board(board) {
  let colors = ["White", "Black"];
  const promise_array = [];
  const image_array = [];
  for (let i = 0; i < colors.length; i++) {
    for (const piece of board[colors[i]]) {
      promise_array.push(new Promise(resolve => {
        var x, y;
  
        [x, y] = get_hexagon_position(piece.rank, piece.file, canvas, hex_size);

        let image_size = hex_size * 1.4;
  
        var image = new Image();
        image.onload = () => {
          ctx.drawImage(image, x - image_size / 2, y - image_size / 2, image_size, image_size);
          resolve();
        };

        image.src = get_piece_asset(colors[i], piece.Piece);
        image_array.push(image);

      }));
    }
  }

  await Promise.all(promise_array); // wait for all images to load
}


function process_clickables(clickable, event, target_size, func) {
  const mouse_x = event.offsetX;
  const mouse_y = event.offsetY;
  for (let i = 0; i < clickable.length; i++) {
    var x, y;
    [x, y] = get_hexagon_position(clickable[i].rank, clickable[i].file, canvas, hex_size);
    if (((mouse_x - x) ** 2 + (mouse_y - y) ** 2) < target_size ** 2) {
      func(clickable[i]);
      break;
    }
  }
}

var selected_piece = null;

var player_color = "White";

function show_available_moves(piece) {
  // send a message to the websocket to get the 
  // valid moves.
  var x, y;
  [x, y] = get_hexagon_position(piece.rank, piece.file, canvas, hex_size);

  if (socket.readyState == socket.OPEN) {
    socket.send(hex_labels[`${x},${y}`]);
  }
  else {
    socket = setup_websocket();
  }
}

function select_piece(piece) {
  selected_piece = piece;
  show_available_moves(piece);
}

function move_piece(destination_hex) {
  console.log("moving piece. send some json", selected_piece, destination_hex);
  draw_board();
  display_board(board);
  if (player_color == "Black") {
    player_color = "White";
  } else {
    player_color = "Black";
  }
}


function handle_click(event) {
  label_hexes(ctx, canvas, hex_size, draw_labels);

  // if we haven't selected a piece, only make pieces valid click targets
  if (selected_piece == null) {
    process_clickables(board[player_color], event, hex_size * 0.866, select_piece);
  }
  
  else {
    process_clickables(valid_moves, event, hex_size * 0.866, move_piece);
    // even if the user clicks an invalid hexagon, deselect the piece
    selected_piece = null;
    draw_board();
    display_board(board);
  }
}

canvas.addEventListener("click", handle_click);

