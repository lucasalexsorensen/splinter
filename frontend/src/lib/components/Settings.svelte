<script lang="ts">
	import { Wifi, Bluetooth } from '@lucide/svelte';

	type Props = {
		connectionMode: 'websocket' | 'bluetooth';
	};

	let { connectionMode = $bindable() }: Props = $props();
</script>

<div class="flex w-full items-center justify-center gap-2">
	<Wifi />
	<input
		id="connection-mode"
		type="checkbox"
		class="toggle toggle-neutral"
		checked={connectionMode === 'bluetooth'}
		onchange={(e) => {
			const val = (e.target as HTMLInputElement).checked;
			connectionMode = val ? 'bluetooth' : 'websocket';
		}}
	/>
	<Bluetooth />
</div>
