#!/usr/bin/env python

import asyncio
import statistics
import tracemalloc

import websockets
from websockets.extensions import permessage_deflate


CLIENTS = 20
INTERVAL = 1 / 10  # seconds

WB, ML = 12, 5

MEM_SIZE = []


async def client(client):
    # Space out connections to make them sequential.
    await asyncio.sleep(client * INTERVAL)

    tracemalloc.start()

    async with websockets.connect(
        "ws://localhost:8765",
        extensions=[
            permessage_deflate.ClientPerMessageDeflateFactory(
                server_max_window_bits=WB,
                client_max_window_bits=WB,
                compress_settings={"memLevel": ML},
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


async def clients():
    await asyncio.gather(*[client(client) for client in range(CLIENTS + 1)])


asyncio.run(clients())


# First connection incurs non-representative setup costs.
del MEM_SIZE[0]

print(f"µ = {statistics.mean(MEM_SIZE) / 1024:.1f} KiB")
print(f"σ = {statistics.stdev(MEM_SIZE) / 1024:.1f} KiB")
