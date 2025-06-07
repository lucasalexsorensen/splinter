import type { Message, MessageType } from './types';

export function deserialize(bytes: Uint8Array): Message {
	const scanner = new Scanner(bytes);
	const type = scanner.readMessageType();

	switch (type) {
		case 'count_updated':
			return {
				type,
				left: scanner.readInt32(),
				right: scanner.readInt32()
			};
		case 'target_updated':
			return {
				type,
				left: scanner.readInt32(),
				right: scanner.readInt32()
			};
		case 'gyro_updated':
			return {
				type,
				x: scanner.readInt16(),
				y: scanner.readInt16(),
				z: scanner.readInt16()
			};
		case 'config_updated':
			return {
				type,
				config: {
					k_p: scanner.readFloat32(),
					k_d: scanner.readFloat32()
				}
			};
		case 'pid_debug':
			return {
				type
			};
	}
}

class Scanner {
	private bytes: Uint8Array;
	private offset: number;

	constructor(bytes: Uint8Array) {
		this.bytes = bytes;
		this.offset = 0;
	}

	readMessageType(): MessageType {
		const type = this.bytes[this.offset];
		this.offset += 1;

		switch (type) {
			case 0x01:
				return 'count_updated';
			case 0x02:
				return 'target_updated';
			case 0x03:
				return 'gyro_updated';
			case 0x04:
				return 'config_updated';
			default:
				throw new Error(`Unknown message type: ${type}`);
		}
	}

	readFloat32(): number {
		const view = new DataView(this.bytes.buffer, this.bytes.byteOffset, this.bytes.byteLength);
		const value = view.getFloat32(this.offset, true);
		this.offset += 4;
		return value;
	}

	readInt32(): number {
		const view = new DataView(this.bytes.buffer, this.bytes.byteOffset, this.bytes.byteLength);
		const value = view.getInt32(this.offset, true);
		this.offset += 4;
		return value;
	}

	readInt16(): number {
		const view = new DataView(this.bytes.buffer, this.bytes.byteOffset, this.bytes.byteLength);
		const value = view.getInt16(this.offset, true);
		this.offset += 2;
		return value;
	}
}
