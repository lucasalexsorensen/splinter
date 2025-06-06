<script lang="ts">
	import {
		BotControls,
		BotSettingsInputs,
		BotStateChart,
		CommandQueue,
		ConnectionCard,
		Settings
	} from '$lib/components';
	import { deserialize } from '$lib/deserialization';
	import {
		WebsocketConnection,
		type IConnection,
		type ConnectionState,
		BluetoothConnection
	} from '$lib/network';
	import { serialize } from '$lib/serialization';
	import { type BotConfig, type Command } from '$lib/types';
	import { onDestroy, onMount } from 'svelte';

	const WS_URL = 'ws://localhost:9999';
	let connectionMode = $state<'websocket' | 'bluetooth'>('bluetooth');
	let connectionState = $state<ConnectionState>('disconnected');

	const buildConnection = (mode: 'websocket' | 'bluetooth'): IConnection => {
		let connection =
			mode === 'websocket' ? new WebsocketConnection(WS_URL) : new BluetoothConnection();
		connection.onStateChange = onStateChange;
		connection.onData = onData;
		return connection;
	};
	let connection: IConnection | null = $state(null);
	onMount(() => {
		connection = buildConnection(connectionMode);
	});

	function onStateChange(state: ConnectionState) {
		connectionState = state;
	}

	function onData(data: Uint8Array) {
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
				break;
			case 'config_updated':
				botSettings = message.config;
				break;
		}
	}

	const isActive = $derived(connectionState === 'connected');
	let botSettings = $state<BotConfig>({ k_p: 0.001, k_d: 0.0002 });
	let leftCounts = $state<number[]>([]);
	let rightCounts = $state<number[]>([]);
	let leftTarget = $state<number>(0);
	let rightTarget = $state<number>(0);
	let gyroX = $state<number[]>([]);
	let gyroY = $state<number[]>([]);
	let gyroZ = $state<number[]>([]);

	onDestroy(() => {
		connection?.close();
	});

	function sendCommand(command: Command) {
		const serialized = serialize(command);
		connection?.write(serialized);
	}
</script>

<div class="flex">
	<div class="flex h-screen w-1/3 flex-col items-center justify-between gap-4 py-8">
		<ConnectionCard
			status={connectionState}
			connect={() => connection?.open() ?? Promise.resolve()}
		/>
		<BotControls disabled={!isActive} {sendCommand} />
		<BotSettingsInputs
			settings={botSettings}
			disabled={!isActive}
			updateSettings={(settings) => {
				const cmd = { type: 'configure', config: settings } as Command;
				sendCommand(cmd);
			}}
		/>
		<Settings bind:connectionMode />
	</div>
	<div class="divider divider-horizontal"></div>
	<div class="flex h-screen w-2/3 flex-col items-center justify-between gap-4 p-4">
		<BotStateChart {leftCounts} {rightCounts} {leftTarget} {rightTarget} />
		<CommandQueue />
	</div>
</div>
