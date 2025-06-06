export type ConnectionState = 'connected' | 'disconnected' | 'connecting' | 'error';

export interface IConnection {
	open(): Promise<void>;
	close(): Promise<void>;
	onData(data: Uint8Array): void;
	onStateChange(state: ConnectionState): void;
	write(data: Uint8Array): Promise<void>;
}

abstract class ConnectionBase implements IConnection {
	abstract open(): Promise<void>;
	abstract close(): Promise<void>;
	abstract write(data: Uint8Array): Promise<void>;
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	onData(data: Uint8Array): void {}
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	onStateChange(state: ConnectionState) {}
}

export class WebsocketConnection extends ConnectionBase {
	private socket?: WebSocket;

	constructor(private url: string) {
		super();
	}

	async open(): Promise<void> {
		// no-op
		this.socket = new WebSocket(this.url);
		this.socket.binaryType = 'arraybuffer';

		this.socket.onopen = () => this.onStateChange('connected');
		this.socket.onclose = () => this.onStateChange('disconnected');
		this.socket.onerror = () => this.onStateChange('error');
		this.socket.onmessage = (event) => this.onData(new Uint8Array(event.data));
	}

	async close(): Promise<void> {
		this.socket?.close();
	}

	async write(data: Uint8Array): Promise<void> {
		this.socket?.send(data);
	}
}

const BT_SERVICE_UUID = 'deadbeef-dead-beef-dead-beefdeadbeef';
const TX_CHARACTERISTIC_UUID = '408813df-5dd4-1f87-ec11-cdb001100000';
const RX_CHARACTERISTIC_UUID = '408813df-5dd4-1f87-ec11-cdb001100001';

export class BluetoothConnection extends ConnectionBase {
	private device?: BluetoothDevice;
	private server?: BluetoothRemoteGATTServer;
	private tx?: BluetoothRemoteGATTCharacteristic;
	private rx?: BluetoothRemoteGATTCharacteristic;
	private lastNotificationTime: number = 0;
	private monitorInterval?: ReturnType<typeof setInterval>;

	constructor() {
		super();
	}

	async open() {
		this.device = await navigator.bluetooth.requestDevice({
			filters: [
				{
					name: 'Rat',
					services: [BT_SERVICE_UUID]
				}
			]
		});
		this.server = await this.device?.gatt?.connect();
		const service = await this.server?.getPrimaryService(BT_SERVICE_UUID);
		this.tx = await service?.getCharacteristic(TX_CHARACTERISTIC_UUID);
		this.rx = await service?.getCharacteristic(RX_CHARACTERISTIC_UUID);
		this.tx?.startNotifications();
		this.tx?.addEventListener('characteristicvaluechanged', (event) => {
			const value = (event.target as BluetoothRemoteGATTCharacteristic)?.value;
			if (value) {
				this.lastNotificationTime = Date.now();
				this.onData(new Uint8Array(value.buffer));
			}
		});

		// Start monitoring notifications
		this.lastNotificationTime = Date.now();
		this.monitorInterval = setInterval(() => {
			const timeSinceLastNotification = Date.now() - this.lastNotificationTime;
			if (timeSinceLastNotification > 1000) {
				console.log('No notifications received in 1000ms, marking connection as failed');
				this.onStateChange('error');
				this.close();
			}
		}, 1000);

		this.onStateChange('connected');
	}

	async close() {
		if (this.monitorInterval) {
			clearInterval(this.monitorInterval);
			this.monitorInterval = undefined;
		}
		this.device?.gatt?.disconnect();
	}

	async write(data: Uint8Array): Promise<void> {
		this.rx?.writeValue(data);
	}
}
