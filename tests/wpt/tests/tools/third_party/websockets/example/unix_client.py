#!/usr/bin/env python

# WS client example connecting to a Unix socket

import asyncio
import os.path
import websockets

async def hello():
    socket_path = os.path.join(os.path.dirname(__file__), "socket")
    async with websockets.unix_connect(socket_path) as websocket:
        name = input("What's your name? ")
        await websocket.send(name)
        print(f"> {name}")

        greeting = await websocket.recv()
        print(f"< {greeting}")

asyncio.get_event_loop().run_until_complete(hello())
