import type { Command } from './types';

export function serialize(command: Command): Uint8Array {
	switch (command.type) {
		case 'turn_left':
			return new Uint8Array([0x01]);
		case 'turn_right':
			return new Uint8Array([0x02]);
		case 'move_forward':
			return new Uint8Array([0x03]);
		case 'move_backward':
			return new Uint8Array([0x04]);
		case 'debug_motors':
			return new Uint8Array([0x05]);
		case 'configure':
			return new Uint8Array([
				0x06,
				...numberToF32LittleEndianBytes(command.config.k_p),
				...numberToF32LittleEndianBytes(command.config.k_d)
			]);
	}
}

function numberToF32LittleEndianBytes(num: number): Uint8Array {
	const buffer = new ArrayBuffer(4); // 4 bytes for f32
	const view = new DataView(buffer);
	view.setFloat32(0, num, true); // true = little-endian
	return new Uint8Array(buffer);
}
