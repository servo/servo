#!/usr/bin/env python

import asyncio
import sys
import websockets


URI = "ws://localhost:32080"


async def run(client_id, messages):
    async with websockets.connect(URI) as websocket:
        for message_id in range(messages):
            await websocket.send(f"{client_id}:{message_id}")
            await websocket.recv()


async def benchmark(clients, messages):
    await asyncio.wait([
        asyncio.create_task(run(client_id, messages))
        for client_id in range(clients)
    ])


if __name__ == "__main__":
    clients, messages = int(sys.argv[1]), int(sys.argv[2])
    asyncio.run(benchmark(clients, messages))
