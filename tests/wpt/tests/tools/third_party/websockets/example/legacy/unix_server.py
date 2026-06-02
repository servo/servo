#!/usr/bin/env python

# WS server example listening on a Unix socket

import asyncio
import os.path
import websockets

async def hello(websocket):
    name = await websocket.recv()
    print(f"<<< {name}")

    greeting = f"Hello {name}!"

    await websocket.send(greeting)
    print(f">>> {greeting}")

async def main():
    socket_path = os.path.join(os.path.dirname(__file__), "socket")
    async with websockets.unix_serve(hello, socket_path):
        await asyncio.Future()  # run forever

asyncio.run(main())
