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

	function handle_incoming_message(message: MessageEvent) {
		const payload = JSON.parse(message.data);
		if (payload.op == 'ValidMoves') {
			valid_moves = payload.moves;
		} else if (payload.op == 'BoardState') {
			board.update(() => instantiate_pieces(payload.board));
			valid_moves = [];
		} else if (payload.op == 'JoinGameSuccess') {
			let session_id = payload.session;
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
			const BACKEND_URL = 'ws://127.0.0.1:7878';
			const socket = new WebSocket(BACKEND_URL);
			let sender = (message: string | ArrayBufferLike | Blob | ArrayBufferView) => socket.send(message);
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

<body>
	<div class="website_id">playhexchess.com</div>
	<div
		bind:clientWidth={board_w}
		bind:clientHeight={board_h}
		class="board"
		style:position="relative"
		style:display="block"
	>
		<img src="/src/assets/board.svg" alt="game board" />
		{#each $board as { hex, position, img_src, alt }}
			<div
				use:draggable={{
					position: {
						x: board_w * (position.x * 0.97 + 0.059 - size * 0.23),
						y: (position.y * 0.99 - 1.59 - size * 0.17) * board_h
					}
				}}
				on:neodrag:start={() => {{selected_piece = hex; show_available_moves(hex, user_id, socket_send)}}}
				on:neodrag:end={() => {if (hover_hex) {move_piece(hex, hover_hex, user_id, socket_send)}; board.update((board) => board)}}
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
				use:draggable={{
					position: {
						x: board_w * (get_hexagon_position(move)[0] * 0.97 + 0.009),
						y: (get_hexagon_position(move)[1] * 0.99 - 1.628) * board_h
					},
					disabled: true
				}}
				on:mouseenter={() => {hover_hex = move}}
				on:mouseleave={() => {hover_hex = null}}
				on:click={() => {move_piece(selected_piece, move, user_id, socket_send); hover_hex = null; valid_moves = [];}}
				style:position="absolute"
				style:width="{board_w * 0.1}px"
				style:height="{board_w * 0.1}px"
			>
				<span 
				class="dot" 
				style:position="relative"
				style:left="{board_w * 0.035}px"
				style:top="{board_w * 0.035}px"
				style:width="{board_w * 0.03}px"
				style:height="{board_w * 0.03}px"/>
			</span>
		{/each}
	</div>
	{#if browser}
		<button
			on:click={socket_send(
				`{"op": "JoinAnyGame",
					"user_id": "${user_id}"}`
			)}
		>
			Join a Multiplayer Game
		</button>
		<button
			on:click={socket_send(
				`{"op": "CreateGame",
				"user_id": "${user_id}",
				"is_multiplayer": false}`
			)}
		>
			Start a Singleplayer Game
		</button>
	{/if}
</body>

<style>
	body {
		background: rgb(22, 20, 49);
		background: linear-gradient(
			283deg,
			rgb(22, 20, 49) 0%,
			rgba(9, 9, 121, 1) 35%,
			rgb(0, 71, 85) 100%
		);
	}
	.website_id {
		color: rgb(255, 255, 255);
		background-color: rgb(0, 0, 0);
		width: 6.6rem;
		padding-left: 0.3rem;
		padding-top: 0.2rem;
		padding-bottom: 0.2rem;
		font-family: Arial, Helvetica, sans-serif;
		font-weight: bolder;
	}
	.board {
		max-height: 120vw;
		max-width: 70vh;
		height: auto;
		width: auto;
		position: relative;
		margin-left: auto;
		margin-right: auto;
	}
	.dot {
		background-color: #a5a195;
		border-radius: 50%;
		display: inline-block;
	}
</style>
