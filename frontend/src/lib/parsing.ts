import type { Command } from './types';

export function parsePayload(bytes: Uint8Array): Message {
	const parser = new Parser(bytes);
	const type = parser.readMessageType();

	switch (type) {
		case 'count_updated':
			return {
				type: 'count_updated',
				left: parser.readInt32(),
				right: parser.readInt32()
			};
		case 'target_updated':
			return {
				type: 'target_updated',
				left: parser.readInt32(),
				right: parser.readInt32()
			};
		case 'queue_updated':
			return {
				type: 'queue_updated',
				commands: []
			};
		case 'gyro_updated':
			return {
				type: 'gyro_updated',
				x: parser.readInt16(),
				y: parser.readInt16(),
				z: parser.readInt16()
			};
	}
}

type MessageType = 'count_updated' | 'target_updated' | 'queue_updated' | 'gyro_updated';

type Message =
	| CountUpdatedMessage
	| TargetUpdatedMessage
	| QueueUpdatedMessage
	| GyroUpdatedMessage;
type CountUpdatedMessage = {
	type: 'count_updated';
	left: number;
	right: number;
};

type TargetUpdatedMessage = {
	type: 'target_updated';
	left: number;
	right: number;
};

type QueueUpdatedMessage = {
	type: 'queue_updated';
	commands: Command[];
};

type GyroUpdatedMessage = {
	type: 'gyro_updated';
	x: number;
	y: number;
	z: number;
};

class Parser {
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
				return 'queue_updated';
			case 0x04:
				return 'gyro_updated';
			default:
				throw new Error(`Unknown message type: ${type}`);
		}
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
