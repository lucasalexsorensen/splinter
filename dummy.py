# /// script
# dependencies = [
#   "websockets",
# ]
# ///

import asyncio
import math
import struct
import time

from websockets.asyncio.server import serve


async def handler(websocket):
    async def send_data():
        x = 0
        while True:
            try:
                v1 = int(math.sin(x) * 1000) & 0xFFFFFFFF  # Keep as 32-bit
                v2 = int(math.cos(x) * 1000) & 0xFFFFFFFF
                data = struct.pack('<BII', 0x01, v1, v2)
                await websocket.send(data)
                x += 0.05
                await asyncio.sleep(0.1)
            except Exception as e:
                print(f"Error sending data: {e}")
                break
    
    send_task = asyncio.create_task(send_data())
    
    try:
        # Handle incoming messages
        while True:
            message = await websocket.recv()
            print(f"Received: {message}")
    except Exception as e:
        print(f"Connection closed: {e}")
    finally:
        send_task.cancel()


async def main():
    async with serve(handler, "", 9999) as server:
        print("WebSocket server started on ws://localhost:9999")
        print("Sending 9 bytes [0x01, i32, i32] every 100ms")
        await server.serve_forever()


if __name__ == "__main__":
    asyncio.run(main())