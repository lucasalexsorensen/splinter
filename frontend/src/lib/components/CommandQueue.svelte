<script lang="ts">
	import { Command } from '$lib/types';
	import { ArrowLeft, Bug, ArrowRight, ArrowUp, ArrowDown } from '@lucide/svelte';
	import type { Component } from 'svelte';
	import { fly } from 'svelte/transition';

	let queue = $state<Command[]>([]);

	function commandToIcon(command: Command): Component {
		switch (command) {
			case Command.TurnLeft:
				return ArrowLeft;
			case Command.TurnRight:
				return ArrowRight;
			case Command.MoveForward:
				return ArrowUp;
			case Command.MoveBackward:
				return ArrowDown;
			case Command.DebugMotors:
				return Bug;
			default:
				throw new Error(`Unknown command: ${command}`);
		}
	}
</script>

<div class="flex h-32 w-full flex-col gap-2 rounded-xl bg-gray-100 p-4">
	<span class="text-sm font-semibold">Queue</span>

	<div class="flex gap-4">
		{#each queue as command}
			{@const Icon = commandToIcon(command)}
			<div class="card card-xs bg-white shadow-sm" in:fly={{ x: 50 }} out:fly={{ x: -50 }}>
				<div class="card-body">
					<Icon />
				</div>
			</div>
		{:else}
			<p>No commands in queue</p>
		{/each}
	</div>
</div>
