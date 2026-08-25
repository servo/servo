#!/usr/bin/env python

import asyncio
import os
import signal
import statistics
import tracemalloc

import websockets
from websockets.extensions import permessage_deflate


CLIENTS = 20
INTERVAL = 1 / 10  # seconds

WB, ML = 12, 5

MEM_SIZE = []


async def handler(ws):
    msg = await ws.recv()
    await ws.send(msg)

    msg = await ws.recv()
    await ws.send(msg)

    MEM_SIZE.append(tracemalloc.get_traced_memory()[0])
    tracemalloc.stop()

    tracemalloc.start()

    # Hold connection open until the end of the test.
    await asyncio.sleep(CLIENTS * INTERVAL)


async def server():
    loop = asyncio.get_running_loop()
    stop = loop.create_future()

    # Set the stop condition when receiving SIGTERM.
    print("Stop the server with:")
    print(f"kill -TERM {os.getpid()}")
    print()
    loop.add_signal_handler(signal.SIGTERM, stop.set_result, None)

    async with websockets.serve(
        handler,
        "localhost",
        8765,
        extensions=[
            permessage_deflate.ServerPerMessageDeflateFactory(
                server_max_window_bits=WB,
                client_max_window_bits=WB,
                compress_settings={"memLevel": ML},
            )
        ],
    ):
        tracemalloc.start()
        await stop


asyncio.run(server())


# First connection may incur non-representative setup costs.
del MEM_SIZE[0]

print(f"µ = {statistics.mean(MEM_SIZE) / 1024:.1f} KiB")
print(f"σ = {statistics.stdev(MEM_SIZE) / 1024:.1f} KiB")
