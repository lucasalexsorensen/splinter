<script lang="ts">
	import { type Command } from '$lib/types';
	import { ArrowLeft, Bug, ArrowRight, ArrowUp, ArrowDown, Settings } from '@lucide/svelte';
	import type { Component } from 'svelte';
	import { fly } from 'svelte/transition';

	let queue = $state<Command[]>([]);

	function commandToIcon(command: Command): Component {
		switch (command.type) {
			case 'turn_left':
				return ArrowLeft;
			case 'turn_right':
				return ArrowRight;
			case 'move_forward':
				return ArrowUp;
			case 'move_backward':
				return ArrowDown;
			case 'debug_motors':
				return Bug;
			case 'configure':
				return Settings;
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
