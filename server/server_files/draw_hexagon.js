import {
  draw_hexagon,
  calc_column_x_position, calc_column_y_positions, vertical_hexagons_per_column, get_hexagon_position, files, char_to_file
} from "./hex_frontend_funcs.js";

const draw_labels = false;

var canvas = document.getElementById("hexagon");
var ctx = canvas.getContext("2d");

var multiplayer_enabled;

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
  ctx.clearRect(0, 0, canvas.width, canvas.height);
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

function parse_moves(moves) {
  valid_moves = moves;
  valid_moves.forEach((val) => { let { rank, file } = parse_hexagon_string(val); draw_dot(rank, file); });
}

var board;

function handle_incoming_message(message) {
  var payload = JSON.parse(message.data);
  if (payload.op == "ValidMoves") {
    parse_moves(payload.moves);
  } else if (payload.op == "BoardState") {
    board = payload.board;
    draw_pieces_from_board_state(payload.board);
  } else if (payload.op == "JoinGameSuccess") {
    console.log(payload);
    document.getElementById("session_displayer").textContent = "Session ID: " + payload.session;
    player_color = payload.color;
  }
}

var hex_labels = label_hexes(ctx, canvas, hex_size, draw_labels);

var user_id = crypto.randomUUID();

function setup_websocket() {
  const BACKEND_URL = "wss://" + window.location.hostname + ":" + window.location.port;
  const socket = new WebSocket(BACKEND_URL);
  socket.onmessage = (message) => handle_incoming_message(message);
  socket.addEventListener("open", () => {console.log("Socket Open"); request_board_state();});
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
};


function get_piece_asset(color, piece) {
  return `assets/pieces/${piece.toLowerCase()}_${color.toLowerCase()}.svg`;
}


// fetch("starting_moves.json").then((res) => res.json()).then(res => board = res).then((res) => display_board(res));

async function get_pieces() {
  const promise_array = [];
  const image_array = {};

  for (var color in Color) {
    for (var piece in Pieces) {
      promise_array.push(new Promise(resolve => {
        var image = new Image();
        image.onload = () => resolve();
        image.src = get_piece_asset(color, piece);
        image_array[`${color},${piece}`] = image;
      }));
    }
  }
  await Promise.all(promise_array); // wait for all images to load
  return image_array;
}

var images = await get_pieces();

function draw_pieces_from_board_state(board) {
  draw_board();
  for (const [position, piece] of Object.entries(board.occupied_squares)) {
    let { rank, file } = parse_hexagon_string(position);

    var x, y;

    [x, y] = get_hexagon_position(rank, file, canvas, hex_size);

    let image_size = hex_size * 1.4;
    let image = images[`${piece.color},${piece.piece_type}`];
    ctx.drawImage(image, x - image_size / 2, y - image_size / 2, image_size, image_size);
  }
}

function parse_hexagon_string(position) {
  let rank = char_to_file[position[0]];
  let file = Number(position.slice(1)) - 1;
  return { rank, file };
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

  // socket.onmessage = (msg) => { parse_moves(msg.data); };

  if (socket.readyState != socket.OPEN) {
    socket = setup_websocket();
  }
  else {
    socket.send(
      `{"op": "GetMoves",
        "user_id": "${user_id}",
        "hexagon": "${hex_labels[`${x},${y}`]}"}`
    );
  }
}

function select_piece(piece) {
  selected_piece = piece;
  show_available_moves(piece);
}

function move_piece(destination_hex) {
  // send a message to register a piece moving
  var dest_x, dest_y;
  [dest_x, dest_y] = get_hexagon_position(destination_hex.rank, destination_hex.file, canvas, hex_size);

  var x, y;
  [x, y] = get_hexagon_position(selected_piece.rank, selected_piece.file, canvas, hex_size);

  //
  // socket.onmessage = (msg) => { draw_board(); draw_pieces_from_board_state(JSON.parse(msg.data).occupied_squares); };


  if (socket.readyState != socket.OPEN) {
    socket = setup_websocket();
  }
  socket.send(
    `{"op": "RegisterMove",
        "user_id": "${user_id}",
        "start_hexagon": "${hex_labels[`${x},${y}`]}",
        "final_hexagon": "${hex_labels[`${dest_x},${dest_y}`]}"}`
  );
  draw_board();
  request_board_state();
  if (!multiplayer_enabled) {
    if (player_color == "Black") {
      player_color = "White";
    } else {
      player_color = "Black";
    }
  }
}

function request_board_state() {
  if (socket.readyState != socket.OPEN) {
    socket = setup_websocket();
  }
  socket.send(
    `{"op": "GetBoard",
      "user_id": "${user_id}"}`
  );
}

function get_matching_board_pieces(board, color) {
  var matching_pieces = {};
  Object.entries(board).forEach(
    ([hexagon, piece]) => { let { rank, file } = parse_hexagon_string(hexagon); piece.color === color ? matching_pieces[hexagon] = { "rank": rank, "file": file } : null; });
  return matching_pieces;
}

function handle_click(event) {
  draw_board();
  draw_pieces_from_board_state(board);
  request_board_state();
  label_hexes(ctx, canvas, hex_size, draw_labels);

  // if we haven't selected a piece, only make pieces valid click targets
  if (selected_piece == null) {
    process_clickables(Object.values(get_matching_board_pieces(board.occupied_squares, player_color)), event, hex_size * 0.866, select_piece);
  }

  else {
    let moves = [];
    valid_moves.forEach((x) => { let { rank, file } = parse_hexagon_string(x); moves.push({ "rank": rank, "file": file }); });
    process_clickables(moves, event, hex_size * 0.866, move_piece);
    // even if the user clicks an invalid hexagon, deselect the piece
    selected_piece = null;
    draw_board();
    draw_pieces_from_board_state(board);
    request_board_state();
  }
}

// set up new games
document.getElementById("NewGame").onclick = () => start_game(false);
document.getElementById("NewMultiplayerGame").onclick = () => start_game(true);

function start_game(is_multiplayer) {
  multiplayer_enabled = is_multiplayer;
  if (socket.readyState != socket.OPEN) {
    socket = setup_websocket();
  }
  else {
    socket.send(
      `{"op": "CreateGame",
        "user_id": "${user_id}",
        "is_multiplayer": ${is_multiplayer}}`
    );
  }
}

function join_game() {
  // join multiplayer
  var session_id = document.getElementById("session_id").value;
  console.log(session_id);
  if (socket.readyState != socket.OPEN) {
    socket = setup_websocket();
  }
  else {
    socket.send(
      `{"op": "JoinGame",
        "user_id": "${user_id}",
        "game_id": "${session_id}"}`
    );
  }
}


document.getElementById("join_session_button").onclick = () => join_game();


canvas.addEventListener("click", handle_click);