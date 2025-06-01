<script lang="ts">
	import {
		BotControls,
		BotSettingsInputs,
		BotStateChart,
		CommandQueue,
		ConnectionCard
	} from '$lib/components';
	import { parsePayload } from '$lib/parsing';
	import { Command, type BotSettings, type ConnectionState } from '$lib/types';
	import { onDestroy, onMount } from 'svelte';

	const WS_URL = 'ws://192.168.1.243:9999';
	let connectionState = $state<ConnectionState>('connecting');
	let ws = $state<WebSocket | null>(null);
	const isActive = $derived(connectionState === 'connected');
	let botSettings = $state<BotSettings>({ k_p: 0.001, k_d: 0.0002 });
	let leftCounts = $state<number[]>([]);
	let rightCounts = $state<number[]>([]);
	let leftTarget = $state<number>(0);
	let rightTarget = $state<number>(0);

	function sendCommand(command: Command) {
		switch (command) {
			case Command.TurnLeft:
				ws?.send(new Uint8Array([0x01]));
				break;
			case Command.TurnRight:
				ws?.send(new Uint8Array([0x02]));
				break;
			case Command.MoveForward:
				ws?.send(new Uint8Array([0x03]));
				break;
			case Command.MoveBackward:
				ws?.send(new Uint8Array([0x04]));
				break;
			case Command.DebugMotors:
				ws?.send(new Uint8Array([0x05]));
				break;
			default:
				throw new Error(`Unknown command: ${command}`);
		}
	}

	function numberToF32LittleEndianBytes(num: number): Uint8Array {
		const buffer = new ArrayBuffer(4); // 4 bytes for f32
		const view = new DataView(buffer);
		view.setFloat32(0, num, true); // true = little-endian
		return new Uint8Array(buffer);
	}

	function connectWebSocket() {
		if (ws) {
			ws.close();
		}

		connectionState = 'connecting';
		ws = new WebSocket(WS_URL);
		ws.binaryType = 'arraybuffer';

		ws.onopen = () => {
			connectionState = 'connected';
		};
		ws.onclose = () => {
			connectionState = 'disconnected';
		};
		ws.onerror = () => {
			connectionState = 'error';
		};
		ws.onmessage = (e) => {
			const payload = new Uint8Array(e.data);
			const message = parsePayload(payload);

			if (message.type === 'count_updated') {
				leftCounts = leftCounts.slice(-50).concat(message.left);
				rightCounts = rightCounts.slice(-50).concat(message.right);
			} else if (message.type === 'target_updated') {
				leftTarget = message.left;
				rightTarget = message.right;
			}
		};
	}

	function retryConnection() {
		connectWebSocket();
	}

	onMount(() => {
		connectWebSocket();
	});

	onDestroy(() => {
		ws?.close();
	});
</script>

<div class="flex">
	<div class="flex h-screen w-1/3 flex-col items-center justify-between gap-4 py-8">
		<ConnectionCard status={connectionState} onRetry={retryConnection} wsUrl={WS_URL} />
		<BotControls disabled={!isActive} {sendCommand} />
		<BotSettingsInputs
			settings={botSettings}
			disabled={!isActive}
			updateSettings={(settings) => {
				console.log('update', settings);
				ws?.send(
					new Uint8Array([
						0x06,
						...numberToF32LittleEndianBytes(settings.k_p),
						...numberToF32LittleEndianBytes(settings.k_d)
					])
				);
			}}
		/>
	</div>
	<div class="divider divider-horizontal"></div>
	<div class="flex h-screen w-2/3 flex-col items-center justify-between gap-4 p-4">
		<BotStateChart {leftCounts} {rightCounts} {leftTarget} {rightTarget} />
		<CommandQueue />
	</div>
</div>
