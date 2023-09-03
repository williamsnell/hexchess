import { board, instantiate_pieces } from './board_state.js';
import { get_hexagon_position } from './get_hexagon_position.js';
import { writable } from "svelte/store";

export function handle_incoming_message(message: MessageEvent) {
	const payload = JSON.parse(message.data);
	if (payload.op == 'ValidMoves') {
		parse_moves(payload.moves);
	} else if (payload.op == 'BoardState') {
		console.log(payload);
		board.update(() => instantiate_pieces(payload.board));
	} else if (payload.op == 'JoinGameSuccess') {
		console.log(payload);
		let session_id = payload.session;
		// recompute the board positions since it may have flipped
	} else if (payload.op == 'GameEnded') {
		console.log(`You ${payload.game_outcome} by ${payload.reason}!`);
	}
}

let valid_moves = [];

function draw_dot_x_y(x, y, radius = 0.3 * hex_size) {
	board_w
	ctx.beginPath();
	ctx.fillStyle = "#000000";
	ctx.lineStyle = "#000000";
	ctx.lineWidth = 0;
	ctx.arc(x, y, radius, 0, 2 * Math.PI);
	ctx.fill();
	ctx.stroke();
	ctx.closePath();
  }

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
