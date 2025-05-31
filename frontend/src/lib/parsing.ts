export function parsePayload(bytes: Uint8Array) {
  // first 4 bytes are little endian i32
  const left_count = readInt32LE(bytes, 0);
  // second 4 bytes are little endian i32
  const right_count = readInt32LE(bytes, 4);
  // third 4 bytes are little endian i32
  const left_target = readInt32LE(bytes, 8);
  // fourth 4 bytes are little endian i32
  const right_target = readInt32LE(bytes, 12);
  return { left_count, right_count, left_target, right_target };
}

function readInt32LE(buffer: Uint8Array, offset: number = 0): number {
  const view = new DataView(
    buffer.buffer,
    buffer.byteOffset,
    buffer.byteLength
  );
  return view.getInt32(offset, true); // true for little endian
}
