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
const TX_CHARACTERISTIC_UUID = 'deadbeef-dead-beef-dead-beefdeadbeef';
const RX_CHARACTERISTIC_UUID = 'deadbeef-dead-beef-dead-beefdeadbeed';

export class BluetoothConnection extends ConnectionBase {
	private device?: BluetoothDevice;
	private server?: BluetoothRemoteGATTServer;
	private tx?: BluetoothRemoteGATTCharacteristic;
	private rx?: BluetoothRemoteGATTCharacteristic;

	constructor() {
		super();
	}

	async open() {
		this.device = await navigator.bluetooth.requestDevice({
			filters: [{ services: [BT_SERVICE_UUID] }],
			optionalServices: [BT_SERVICE_UUID]
		});
		this.server = await this.device?.gatt?.connect();
		const service = await this.server?.getPrimaryService(BT_SERVICE_UUID);
		this.tx = await service?.getCharacteristic(TX_CHARACTERISTIC_UUID);
		this.rx = await service?.getCharacteristic(RX_CHARACTERISTIC_UUID);
		this.onStateChange('connected');

		this.tx?.addEventListener('characteristicvaluechanged', (event) => {
			const value = (event.target as BluetoothRemoteGATTCharacteristic)?.value;
			if (value) {
				this.onData(new Uint8Array(value.buffer));
			}
		});
	}

	async close() {
		this.device?.gatt?.disconnect();
	}

	async write(data: Uint8Array): Promise<void> {
		this.rx?.writeValue(data);
	}
}
