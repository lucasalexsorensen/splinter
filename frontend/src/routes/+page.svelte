<script lang="ts">
	import {
		BotControls,
		BotSettingsInputs,
		BotStateChart,
		CommandQueue,
		ConnectionCard
	} from '$lib/components';
	import { deserialize } from '$lib/deserialization';
	import { WebsocketConnection, type IConnection, type ConnectionState } from '$lib/network';
	import { serialize } from '$lib/serialization';
	import { type BotSettings, type Command } from '$lib/types';
	import { onDestroy, onMount } from 'svelte';

	const WS_URL = 'ws://localhost:9999';
	let connectionState = $state<ConnectionState>('connecting');
	let connection: IConnection = new WebsocketConnection(WS_URL);
	connection.onStateChange = (state: ConnectionState) => {
		connectionState = state;
	};
	connection.onData = (data: Uint8Array) => {
		const message = deserialize(data);
		switch (message.type) {
			case 'count_updated':
				leftCounts = leftCounts.slice(-99).concat(message.left);
				rightCounts = rightCounts.slice(-99).concat(message.right);
				break;
			case 'target_updated':
				leftTarget = message.left;
				rightTarget = message.right;
				break;
			case 'gyro_updated':
				gyroX = gyroX.slice(-99).concat(message.x);
				gyroY = gyroY.slice(-99).concat(message.y);
				gyroZ = gyroZ.slice(-99).concat(message.z);
		}
	};

	const isActive = $derived(connectionState === 'connected');
	let botSettings = $state<BotSettings>({ k_p: 0.001, k_d: 0.0002 });
	let leftCounts = $state<number[]>([]);
	let rightCounts = $state<number[]>([]);
	let leftTarget = $state<number>(0);
	let rightTarget = $state<number>(0);
	let gyroX = $state<number[]>([]);
	let gyroY = $state<number[]>([]);
	let gyroZ = $state<number[]>([]);

	onMount(() => {
		connection.open();
	});

	onDestroy(() => {
		connection.close();
	});

	function sendCommand(command: Command) {
		const serialized = serialize(command);
		connection.write(serialized);
	}
</script>

<div class="flex">
	<div class="flex h-screen w-1/3 flex-col items-center justify-between gap-4 py-8">
		<ConnectionCard status={connectionState} onRetry={() => connection.open()} wsUrl={WS_URL} />
		<BotControls disabled={!isActive} {sendCommand} />
		<BotSettingsInputs
			settings={botSettings}
			disabled={!isActive}
			updateSettings={(settings) => {
				const cmd = { type: 'configure', settings } as Command;
				sendCommand(cmd);
			}}
		/>
	</div>
	<div class="divider divider-horizontal"></div>
	<div class="flex h-screen w-2/3 flex-col items-center justify-between gap-4 p-4">
		<BotStateChart {leftCounts} {rightCounts} {leftTarget} {rightTarget} />
		<CommandQueue />
	</div>
</div>
