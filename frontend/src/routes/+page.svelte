<script lang="ts">
	import { get_piece_asset } from './assets.js';
	import { board } from './board_state.js';
	import { Color, PieceType } from './hexchess_logic.js';
	import {draggable} from '@neodrag/svelte';

	let size = 200;

	let piece_images: Array<string> = [];
	for (const color in Color) {
		for (const piece in PieceType) {
			piece_images.push(get_piece_asset(color, piece));
		}
	}

	let board_w, board_h;

</script>

<svelte:head>
	{#each piece_images as image}
		<link rel="preload" as="image" href={image} />
	{/each}
</svelte:head>

<body>
	<div class="website_id">playhexchess.com</div>
	<div bind:clientWidth={board_w} bind:clientHeight={board_h}
		class="board"
		style:position="relative"
		style:display="block">
		<img
			src="/src/assets/board.svg"
			alt="game board"
		/>
		{#each $board as { hex, position, img_src, alt }}
		<div			
			use:draggable={{position: {x: board_w * (position.x * 0.97 + 0.059), y: (position.y * 0.99 - 1.58) *  board_h}}}
			on:neodrag={(e) => {
				$socket.send(
				`{"op": "GetMoves",
					"user_id": "${user_id}",
					"hexagon": "${hex_labels[`${x},${y}`]}"}`
				);
			}}
			style:position="absolute"
		>
			<input
				type="image"
				src={img_src}
				style:left="0px"
				style:top="0px"
				style:transform="translate(-50%, -50%)"
				style:width="{size}%"
				{alt}/>
		</div>
		{/each}
	</div>
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
</style>
