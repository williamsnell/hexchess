<script lang="ts">
	import { get_piece_asset } from './assets.js';
	import { board, instantiate_pieces, show_available_moves, move_piece } from './board_state.js';
	import { Color, PieceType } from './hexchess_logic.js';
	import { draggable } from '@neodrag/svelte';
	import { browser } from '$app/environment';
	import { get_hexagon_position } from './get_hexagon_position.js';

	$: valid_moves = [];
	$: board_w = 0;
	$: board_h = 0;
	$: session_id = 0;

	$: player_color = "Both";
	$: current_player = "";

	$: board_rotate = "auto";

	$: orient = 1;

	
	function choose_orientation(player_color, current_player, board_rotate) {
		if (board_rotate == "auto") {
			if (player_color == "White") {
				orient = 1;
			} else if (player_color == "Black") {
				orient = -1;
			} else if (player_color == "Both") {
				if (current_player == "White") {
					orient = 1;
				} else {
					orient = -1;
				};
			}
		} else if (board_rotate == "White") {
			orient = 1;
		} else {
			orient = -1;
		}
	}

	$: choose_orientation(player_color, current_player, board_rotate);


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
			// choose_orientation();
			// recompute the board positions since it may have flipped
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

	let size = 0.22;

	let piece_images: Array<string> = [];
	for (const color in Color) {
		for (const piece in PieceType) {
			piece_images.push(get_piece_asset(color, piece));
		}
	}

	let hover_hex;
	let selected_piece;
</script>

<svelte:head>
	{#each piece_images as image}
		<link rel="preload" as="image" href={image} />
	{/each}
</svelte:head>

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
	<div bind:offsetWidth={board_w} bind:offsetHeight={board_h} class="board">
		<img src="/assets/board.svg" alt="game board" />
		{#each $board as { hex, position, img_src, alt }}
			<div
				class="piece"
				use:draggable={{
					position: {
						x:
							board_w *
							((-orient * position.x - (0.906 * (1 - orient)) / 2) * 0.97 +
								0.94 -
								size * 0.23),
						y:
							((position.y - (2.18 * (1 - orient)) / 2) * -orient * 0.99 +
								0.57 -
								size * 0.17) *
							board_h
					}
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
			>
				<input
					type="image"
					src={img_src}
					style:left="0px"
					style:top="0px"
					style:width="{size * board_w}%"
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
					position: {
						x:
							board_w *
							((-orient * get_hexagon_position(move)[0] - (0.906 * (1 - orient)) / 2) *
								0.97 +
								0.94 -
								size * 0.23),
						y:
							((get_hexagon_position(move)[1] - (2.18 * (1 - orient)) / 2) *
								-orient *
								0.99 +
								0.57 -
								size * 0.17) *
							board_h
					},
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
				style:width="{board_w * 0.1}px"
				style:height="{board_w * 0.1}px"
			>
				<span
					class="dot"
					style:position="relative"
					style:left="{board_w * 0.035}px"
					style:top="{board_w * 0.035}px"
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
			if (board_rotate == "auto") {
				board_rotate = "White";
			} else if (board_rotate == "White") {
				board_rotate = "Black";
			} else if (board_rotate == "Black") {
				board_rotate = "auto";
			}
			// choose_orientation();
		}
		}
		>
		<h4>Rotate:</h4>
		<p>{board_rotate}</p></button>
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
		padding: -5%;
		margin-bottom: 1rem;
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
