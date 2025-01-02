#!/usr/bin/env python

import asyncio
import signal
import websockets

async def echo(websocket):
    async for message in websocket:
        await websocket.send(message)

async def server():
    # Set the stop condition when receiving SIGTERM.
    loop = asyncio.get_running_loop()
    stop = loop.create_future()
    loop.add_signal_handler(signal.SIGTERM, stop.set_result, None)

    async with websockets.serve(echo, "localhost", 8765):
        await stop

asyncio.run(server())
