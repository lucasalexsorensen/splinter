<script lang="ts">
	import type { BotConfig } from '$lib/types';

	type Props = {
		settings?: BotConfig;
		disabled?: boolean;
		updateSettings: (settings: BotConfig) => void;
	};

	let { settings = $bindable(), disabled = false, updateSettings }: Props = $props();

	// Create reactive local variables that sync with settings
	let k_p = $state(0);
	let k_d = $state(0);

	// Sync local variables with settings when settings change
	$effect(() => {
		k_p = settings?.k_p ?? 0;
		k_d = settings?.k_d ?? 0;
	});

	// Update settings when local variables change
	$effect(() => {
		if (settings) {
			updateSettings({
				...settings,
				k_p,
				k_d
			});
		}
	});
</script>

<div class="flex w-full flex-col gap-4 p-8">
	<div class="flex items-center gap-2">
		<label for="k_p">K<sub>P</sub></label>
		<input
			{disabled}
			id="k_p"
			type="range"
			min="0.0001"
			max="0.1"
			step="0.0001"
			bind:value={k_p}
			class="range range-primary flex-1"
		/>
		<input
			{disabled}
			type="number"
			min="0.0001"
			max="0.1"
			step="0.0001"
			value={k_p.toFixed(4)}
			oninput={(e) => (k_p = parseFloat(e.currentTarget.value))}
			class="input input-sm w-30"
		/>
	</div>
	<div class="flex items-center gap-2">
		<label for="k_d">K<sub>D</sub></label>
		<input
			{disabled}
			id="k_d"
			type="range"
			min="0.000"
			max="0.1"
			step="0.0001"
			bind:value={k_d}
			class="range range-primary flex-1"
		/>
		<input
			{disabled}
			type="number"
			min="0.000"
			max="0.1"
			step="0.0001"
			value={k_d.toFixed(4)}
			oninput={(e) => (k_d = parseFloat(e.currentTarget.value))}
			class="input input-sm w-30"
		/>
	</div>
</div>
