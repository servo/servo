#!/usr/bin/env python

import asyncio
import statistics
import tracemalloc

import websockets
from websockets.extensions import permessage_deflate


CLIENTS = 10
INTERVAL = 1 / 10  # seconds

MEM_SIZE = []


async def mem_client(client):
    # Space out connections to make them sequential.
    await asyncio.sleep(client * INTERVAL)

    tracemalloc.start()

    async with websockets.connect(
        "ws://localhost:8765",
        extensions=[
            permessage_deflate.ClientPerMessageDeflateFactory(
                server_max_window_bits=10,
                client_max_window_bits=10,
                compress_settings={"memLevel": 3},
            )
        ],
    ) as ws:
        await ws.send("hello")
        await ws.recv()

        await ws.send(b"hello")
        await ws.recv()

        MEM_SIZE.append(tracemalloc.get_traced_memory()[0])
        tracemalloc.stop()

        # Hold connection open until the end of the test.
        await asyncio.sleep(CLIENTS * INTERVAL)


asyncio.get_event_loop().run_until_complete(
    asyncio.gather(*[mem_client(client) for client in range(CLIENTS + 1)])
)

# First connection incurs non-representative setup costs.
del MEM_SIZE[0]

print(f"µ = {statistics.mean(MEM_SIZE) / 1024:.1f} KiB")
print(f"σ = {statistics.stdev(MEM_SIZE) / 1024:.1f} KiB")
