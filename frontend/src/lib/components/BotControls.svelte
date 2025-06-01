<script lang="ts">
	import { Command } from '$lib/types';
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
			onclick={() => sendCommand(Command.TurnLeft)}
		>
			<ArrowLeft />
		</button>
		<button
			class="btn btn-soft btn-primary"
			{disabled}
			onclick={() => sendCommand(Command.TurnRight)}
		>
			<ArrowRight />
		</button>
	</div>
	<div class="flex w-full gap-4">
		<button
			class="btn btn-soft btn-primary"
			{disabled}
			onclick={() => sendCommand(Command.MoveForward)}
		>
			<ArrowUp />
		</button>
		<button
			class="btn btn-soft btn-primary"
			{disabled}
			onclick={() => sendCommand(Command.MoveBackward)}
		>
			<ArrowDown />
		</button>
	</div>

	<div class="flex w-full justify-center">
		<button
			class="btn btn-soft btn-primary"
			{disabled}
			onclick={() => sendCommand(Command.DebugMotors)}
		>
			<Bug />
		</button>
	</div>
</div>
