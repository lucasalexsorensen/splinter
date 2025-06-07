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
	import { type BotConfig, type Command, type Message } from '$lib/types';
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
		handleMessage(message);
	}

	function handleMessage(message: Message): boolean {
		switch (message.type) {
			case 'count_updated':
				leftCounts = leftCounts.slice(-99).concat(message.left);
				rightCounts = rightCounts.slice(-99).concat(message.right);
				return true;
			case 'target_updated':
				leftTarget = message.left;
				rightTarget = message.right;
				return true;
			case 'gyro_updated':
				gyroX = gyroX.slice(-99).concat(message.x);
				gyroY = gyroY.slice(-99).concat(message.y);
				gyroZ = gyroZ.slice(-99).concat(message.z);
				return true;
			case 'config_updated':
				console.log('config_updated', message);
				botSettings = message.config;
				return true;
			case 'pid_debug':
				console.log('pid_debug', message);
				return false;
		}
	}

	const isActive = $derived(connectionState === 'connected');
	let botSettings = $state<BotConfig | undefined>(undefined);
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

	function resetState() {
		leftCounts = [];
		rightCounts = [];
		leftTarget = 0;
		rightTarget = 0;
		gyroX = [];
		gyroY = [];
		gyroZ = [];
	}

	function sendCommand(command: Command) {
		const serialized = serialize(command);
		connection?.write(serialized);
	}
</script>

<div class="flex">
	<div class="flex h-screen w-1/3 flex-col items-center justify-between gap-4 py-8">
		<ConnectionCard
			status={connectionState}
			connect={async () => {
				await connection?.open();
				resetState();
			}}
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
		<!-- <CommandQueue /> -->
	</div>
</div>
