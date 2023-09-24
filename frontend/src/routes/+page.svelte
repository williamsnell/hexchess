<script lang="ts">
	import { get_piece_asset } from './assets.js';
	import {
		board,
		Board,
		instantiate_pieces,
		show_available_moves,
		move_piece
	} from './board_state.js';
	import { Color, PieceType } from './hexchess_logic.js';
	import { draggable } from '@neodrag/svelte';
	import { browser } from '$app/environment';
	import { get_hexagon_position } from './get_hexagon_position.js';
 
	import pkg from 'lodash';
	const { isEmpty, transform, isEqual, isArray, isObject } = pkg;

	$: valid_moves = [];
	$: board_w = 0;
	$: board_h = 0;
	$: session_id = 0;
	$: previous_board = [];

	$: player_color = 'Both';
	$: current_player = '';

	$: board_rotate = 'auto';

	$: orient = 1;
	$: last_move = {};

	let size = 0.08;

	function position_to_screenspace(
		x_fraction: number,
		y_fraction: number,
		board_w: number,
		board_h: number,
		orient: number
	) {
		return {
			// x: board_w * (-orient * x_fraction * 0.974 - (1 - orient) * 0.442 + 0.938),
			// y: (y_fraction * orient * -0.998 + (1 - orient) * -1.088 + 0.566) * board_h
			x: board_w * (orient * x_fraction + 0.5),
			y: board_h * (-orient * y_fraction - 0.5)			
		};
	}
 
	function choose_orientation(player_color: string, current_player: string, board_rotate: string) {
		if (board_rotate == 'auto') {
			if (player_color == 'White') {
				orient = 1;
			} else if (player_color == 'Black') {
				orient = -1;
			} else if (player_color == 'Both') {
				if (current_player == 'White') {
					orient = 1;
				} else {
					orient = -1;
				}
			}
		} else if (board_rotate == 'White') {
			orient = 1;
		} else {
			orient = -1;
		}
	}

	$: choose_orientation(player_color, current_player, board_rotate);

	function difference(origObj, newObj) {
		function changes(newObj, origObj) {
			let arrayIndexCounter = 0;
			return transform(newObj, function (result, value, key) {
				if (key != 'position' && !isEqual(value, origObj[key])) {
					let resultKey = isArray(origObj) ? arrayIndexCounter++ : key;
					result[resultKey] =
						isObject(value) && isObject(origObj[key]) ? changes(value, origObj[key]) : value;
				}
			});
		}
		return changes(newObj, origObj);
	}

	function sort_object_by_keys(object) {
		return Object.keys(object)
			.sort()
			.reduce((obj, key) => {
				obj[key] = object[key];
				return obj;
			}, {});
	}

	function get_last_move(board) {
		// get the hex and alt (piece and color)
		let new_pieces = {};
		$board.forEach((val) => (new_pieces[val.hex] = val.alt));

		let old_pieces = {};
		previous_board.forEach((val) => (old_pieces[val.hex] = val.alt));

		// console.log(previous_board);

		// sort everything
		old_pieces = sort_object_by_keys(old_pieces);
		new_pieces = sort_object_by_keys(new_pieces);

		// console.log(old_pieces, "\n\n", new_pieces);

		let delta = difference(old_pieces, new_pieces);
		console.log(delta);
		if (!isEmpty(delta)) {
			if (Object.keys(delta).length == 1) {
				last_move = delta;
			} else {
				last_move = [];
			}
			previous_board = structuredClone($board);
		}
	}

	$: $board, get_last_move(board);

	function handle_incoming_message(message: MessageEvent) {
		const payload = JSON.parse(message.data);
		if (payload.op == 'ValidMoves') {
			valid_moves = payload.moves;
		} else if (payload.op == 'BoardState') {
			current_player = payload.board.current_player;
			// choose_orientation();

			board.update(() => instantiate_pieces(payload.board));
			valid_moves = [];
		} else if (payload.op == 'JoinGameSuccess') {
			session_id = payload.session;
			player_color = payload.color;
		} else if (payload.op == 'GameEnded') {
			console.log(`You ${payload.game_outcome} by ${payload.reason}!`);
		}
	}

	function try_reconnect(send: Function) {
		send(
			`{"op": "TryReconnect",
		"user_id": "${user_id}"}`
		);
	}

	function request_board_state(send: Function) {
		send(
			`{"op": "GetBoard",
		"user_id": "${user_id}"}`
		);
	}

	function setup_socket() {
		// return a function that lets you send messages over the socket
		if (browser) {
			const BACKEND_URL =
				window.location.protocol == 'http:'
					? 'ws://127.0.0.1:7878/ws'
					: 'wss://playhexchess.com:443/ws';
			const socket = new WebSocket(BACKEND_URL);
			let sender = (message: string | ArrayBufferLike | Blob | ArrayBufferView) =>
				socket.send(message);
			socket.onmessage = (message) => handle_incoming_message(message);
			socket.addEventListener('open', () => {
				console.log('Socket Open');
				try_reconnect(sender);
				request_board_state(sender);
			});
			socket.onerror = (err) => console.error(err);
			socket.onclose = () => console.log('Socket Closed');
			return sender;
		} else {
			return () => {};
		}
	}

	const socket_send = setup_socket();

	let user_id = '';

	if (browser) {
		// try get a stored user_id for a session. If there isn't one, make one
		if (sessionStorage.getItem('player_id') != null) {
			user_id = sessionStorage.getItem('player_id');
		} else {
			user_id = crypto.randomUUID();
			sessionStorage.setItem('player_id', user_id);
		}
	}

	let piece_images: Array<string> = [];
	for (const color in Color) {
		for (const piece in PieceType) {
			piece_images.push(get_piece_asset(color, piece));
		}
	}

	let hover_hex: string;
	let selected_piece: string;
</script>

<title>Hexagonal Chessagonal</title>
<body>
	<div class="top-bar">
		<div class="website_id">playhexchess.com</div>
		<div class="github">
			<a href="https://github.com/williamsnell/hexchess">
				<input type="image" src="/assets/github-mark-white.svg" alt="github link" class="github" />
			</a>
		</div>
	</div>
	<div
		id="menu"
		style:text-align="center"
		style:height={session_id == 0 ? '4rem' : '2.2rem'}
		style:width={session_id == 0 ? '100%' : '16.4rem'}
		class="top-menu"
	>
		{#if browser}
			<button
				class="button"
				style:height={session_id == 0 ? '4rem' : '2rem'}
				on:click={socket_send(
					`{"op": "JoinAnyGame",
						"user_id": "${user_id}"}`
				)}
				style:width={session_id == 0 ? '49%' : '8rem'}
				style:font-size={session_id == 0 ? '1.5rem' : '1rem'}
			>
				Multiplayer
			</button>
			<button
				class="button"
				style:height={session_id == 0 ? '4rem' : '2rem'}
				on:click={socket_send(
					`{"op": "CreateGame",
					"user_id": "${user_id}",
					"is_multiplayer": false}`
				)}
				style:width={session_id == 0 ? '49%' : '8rem'}
				style:font-size={session_id == 0 ? '1.5rem' : '1rem'}
			>
				Singleplayer
			</button>
		{/if}
	</div>
	<div bind:clientWidth={board_w} bind:clientHeight={board_h} class="board">
		<img src="/assets/board.svg" alt="game board" style:display="block"/>
		{#if !isEmpty(last_move)}
			<span
				use:draggable={{
					position: position_to_screenspace(
						get_hexagon_position(Object.keys(last_move)[0])[0],
						get_hexagon_position(Object.keys(last_move)[0])[1],
						board_w,
						board_h,
						orient
					),
					disabled: true
				}}
				style:position="absolute"
				style:display="block"
			>
				<img
					src="/assets/highlight.svg"
					alt="highlighted hexagon"
					style:position="relative"
					style:display="block"
					style:top="{-board_w * 0.06}px"
					style:left="{-board_w * 0.06}px"
					style:width="{board_w * 0.1195}px"
					style:height="{board_w * 0.1195}px"
				/>
			</span>
		{/if}
		{#each $board as { hex, position, img_src: src, alt }}
			<div
				class="piece"
				use:draggable={{
					position: position_to_screenspace(position.x, position.y, board_w, board_h, orient)
				}}
				on:pointerdown={(e) => {
					e.target.releasePointerCapture(e.pointerId);
				}}
				on:neodrag:start={() => {
					{
						selected_piece = hex;
						show_available_moves(hex, user_id, socket_send);
					}
				}}
				on:neodrag:end={() => {
					if (hover_hex) {
						move_piece(hex, hover_hex, user_id, socket_send);
					}
					board.update((board) => board);
				}}
				style:position="absolute"
				style:display="block"
				style:width="{board_h * size}px"
				style:left="-{0.5/11.3 * board_w}px"
				style:bottom="{-0.5 / 11 * 100}%"
			>
				<input
					type="image"
					style:display="block"
					src="{src}#svgView(viewBox(3, 10, 39, 28))"
					style:width="100%"
					style:height="100%"
					{alt}
				/>
			</div>
		{/each}
		<!-- draw the valid moves -->
		{#each valid_moves as move}
			<!-- svelte-ignore a11y-no-static-element-interactions -->
			<!-- svelte-ignore a11y-click-events-have-key-events -->
			<!-- svelte-ignore a11y-mouse-events-have-key-events -->
			<span
				style:touch-action="none"
				use:draggable={{
					position: position_to_screenspace(
						get_hexagon_position(move)[0],
						get_hexagon_position(move)[1],
						board_w,
						board_h,
						orient
					),
					disabled: true
				}}
				on:pointerenter={() => {
					hover_hex = move;
				}}
				on:pointerleave={() => {
					hover_hex = null;
				}}
				on:click={() => {
					move_piece(selected_piece, move, user_id, socket_send);
					hover_hex = null;
					valid_moves = [];
				}}
				style:position="absolute"
				style:display="block"
				style:width="9%"
				style:height="8.5%"
				style:left="-4.75%"
				style:bottom="-4.5%"
				style:border-radius="50%"
			>
				<span
					class="dot"
					style:position="relative"
					style:display="block"
					style:left="35%"
					style:top="35%"
					style:width="{board_w * 0.03}px"
					style:height="{board_w * 0.03}px"
				/>
			</span>
		{/each}
	</div>
	<div class="flip_button">
		<button
			class="flip_button"
			on:click={() => {
				if (board_rotate == 'auto') {
					board_rotate = 'White';
				} else if (board_rotate == 'White') {
					board_rotate = 'Black';
				} else if (board_rotate == 'Black') {
					board_rotate = 'auto';
				}
				// choose_orientation();
			}}
		>
			<h4>Rotate:</h4>
			<p>{board_rotate}</p></button
		>
	</div>
</body>

<style>
	body {
		background: rgb(66, 64, 92);
	}
	.top-bar {
		display: flex;
	}
	.website_id {
		color: aliceblue;
		background-color: rgb(0, 0, 0);
		width: 6.6rem;
		height: 1rem;
		padding-left: 0.3rem;
		padding-top: 0.2rem;
		padding-bottom: 0.2rem;
		font-family: Arial, Helvetica, sans-serif;
		font-weight: bolder;
		margin-bottom: 1rem;
	}
	.github {
		position: flex;
		margin-left: auto;
		margin-right: 0;
		width: 1rem;
		height: 1rem;
		background: rgb(0, 0, 0, 0.3);
		border-radius: 1rem;
		transition-duration: 0.6s;
	}
	.github:hover {
		background: rgb(0, 0, 0);
		transition-duration: 0.6s;
	}
	.board {
		max-height: 120vw;
		max-width: calc(89vh - 12rem);
		height: auto;
		width: auto;
		margin-left: auto;
		margin-right: auto;
	}
	.dot {
		background-color: #a5a195;
		border-radius: 50%;
		display: inline-block;
		touch-action: none;
	}
	.piece {
		touch-action: none;
	}
	.button {
		transition-duration: 0.5s;
		margin-left: auto;
		margin-right: auto;
		border-radius: 2rem;
	}
	.button:active {
		background: rgb(0, 0, 0, 0.2);
		transition-duration: 0.2s;
		color: aliceblue;
	}
	.button:hover {
		background: rgba(255, 255, 255, 0.4);
		transition-duration: 0.2s;
	}
	.flip_button {
		position: flex;
		transition-duration: 0.5s;
		margin-left: auto;
		margin-right: 5%;
		margin-top: -5%;
		border-radius: 10vw;
		font-size: calc(min(1rem, 3vw));
		width: calc(min(5rem, 15vw));
		height: calc(min(7rem, 20vw));
	}
	.flip_button:active {
		background: rgb(0, 0, 0, 0.2);
		transition-duration: 0.2s;
		color: aliceblue;
	}
	.flip_button:hover {
		background: rgba(255, 255, 255, 0.4);
		transition-duration: 0.2s;
	}
	.top-menu {
		margin-top: 1%;
		margin-bottom: 1%;
		border-radius: 2rem;
		background: rgb(0, 0, 0, 0.2);
		transition-duration: 0.5s;
		display: flex;
		align-items: center;
	}
</style>
