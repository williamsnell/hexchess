import { board, parse_hexagon_string } from './board_state.js';
import { get_hexagon_position } from './get_hexagon_position.js';
import { writable } from "svelte/store";

export function setup_websocket() {
	const BACKEND_URL =
		(window.location.protocol == 'http:' ? 'ws://' : 'wss://') +
		'127.0.0.1' +
		':' +
		window.location.port;
	const socket = new WebSocket(BACKEND_URL);
	socket.onmessage = (message) => handle_incoming_message(message);
	socket.addEventListener('open', () => {
		console.log('Socket Open');
		try_reconnect();
		request_board_state();
	});
	socket.onerror = (err) => console.error(err);
	socket.onclose = () => console.log('Socket Closed');

	return socket;
}

export const socket = writable(setup_websocket(), () => { });

let user_id: string;
// try get a stored user_id for a session. If there isn't one, make one
if (sessionStorage.getItem('player_id') != null) {
	user_id = sessionStorage.getItem('player_id');
} else {
	user_id = crypto.randomUUID();
	sessionStorage.setItem('player_id', user_id);
}

function try_reconnect() {
	socket.send(
		`{"op": "TryReconnect",
"user_id": "${user_id}"}`
	);
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

function handle_incoming_message(message: MessageEvent) {
	const payload = JSON.parse(message.data);
	if (payload.op == 'ValidMoves') {
		parse_moves(payload.moves);
	} else if (payload.op == 'BoardState') {
		board.update(payload.board);
	} else if (payload.op == 'JoinGameSuccess') {
		console.log(payload);
		let session_id =
			window.location.protocol + '//' + window.location.hostname + '/join?' + payload.session;
		// recompute the board positions since it may have flipped
	} else if (payload.op == 'GameEnded') {
		window.alert(`You ${payload.game_outcome} by ${payload.reason}!`);
	}
}

let valid_moves = [];

// function draw_dot_x_y(x, y, radius = 0.3 * hex_size) {
// 	ctx.beginPath();
// 	ctx.fillStyle = '#000000';
// 	ctx.lineStyle = '#000000';
// 	ctx.lineWidth = 0;
// 	ctx.arc(x, y, radius, 0, 2 * Math.PI);
// 	ctx.fill();
// 	ctx.stroke();
// 	ctx.closePath();
// }

function draw_dot(position) {
	let [x, y] = get_hexagon_position(position);
	draw_dot_x_y(x, y);
}

function parse_moves(moves: Array[]) {
	valid_moves = moves;
	valid_moves.forEach((val) => {
		draw_dot(val);
	});
}
