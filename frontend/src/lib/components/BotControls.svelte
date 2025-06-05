<script lang="ts">
	import { type Command } from '$lib/types';
	import { ArrowLeft, ArrowRight, ArrowUp, ArrowDown, Bug } from '@lucide/svelte';

	type Props = {
		disabled?: boolean;
		sendCommand: (command: Command) => void;
	};

	let { disabled = false, sendCommand }: Props = $props();
</script>

<div class="flex flex-col gap-4">
	<div class="flex w-full gap-4">
		<button
			class="btn btn-soft btn-primary"
			{disabled}
			onclick={() => sendCommand({ type: 'turn_left' })}
		>
			<ArrowLeft />
		</button>
		<button
			class="btn btn-soft btn-primary"
			{disabled}
			onclick={() => sendCommand({ type: 'turn_right' })}
		>
			<ArrowRight />
		</button>
	</div>
	<div class="flex w-full gap-4">
		<button
			class="btn btn-soft btn-primary"
			{disabled}
			onclick={() => sendCommand({ type: 'move_forward' })}
		>
			<ArrowUp />
		</button>
		<button
			class="btn btn-soft btn-primary"
			{disabled}
			onclick={() => sendCommand({ type: 'move_backward' })}
		>
			<ArrowDown />
		</button>
	</div>

	<div class="flex w-full justify-center">
		<button
			class="btn btn-soft btn-primary"
			{disabled}
			onclick={() => sendCommand({ type: 'debug_motors' })}
		>
			<Bug />
		</button>
	</div>
</div>
