<script lang="ts">
	import type { BotSettings } from '$lib/types';

	type Props = {
		settings: BotSettings;
		disabled?: boolean;
		updateSettings: (settings: BotSettings) => void;
	};

	let { settings = $bindable(), disabled = false, updateSettings }: Props = $props();
</script>

<div class="flex w-full flex-col gap-4 p-8">
	<div class="flex items-center gap-2">
		<label for="k_p">K<sub>P</sub></label>
		<input
			{disabled}
			id="k_p"
			type="range"
			min="0.0001"
			max="0.01"
			step="0.0001"
			onchange={(e) =>
				updateSettings({
					...settings,
					k_p: parseFloat((e.target as HTMLInputElement).value)
				})}
			bind:value={settings.k_p}
			class="range range-primary"
		/>
		<span>{settings.k_p.toFixed(4)}</span>
	</div>
	<div class="flex items-center gap-2">
		<label for="k_d">K<sub>D</sub></label>
		<input
			{disabled}
			id="k_d"
			type="range"
			min="0.000"
			max="0.01"
			step="0.0001"
			onchange={(e) =>
				updateSettings({
					...settings,
					k_d: parseFloat((e.target as HTMLInputElement).value)
				})}
			bind:value={settings.k_d}
			class="range range-primary"
		/>
		<span>{settings.k_d.toFixed(4)}</span>
	</div>
</div>
