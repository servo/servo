#!/usr/bin/env python

# WS server example listening on a Unix socket

import asyncio
import os.path
import websockets

async def hello(websocket, path):
    name = await websocket.recv()
    print(f"< {name}")

    greeting = f"Hello {name}!"

    await websocket.send(greeting)
    print(f"> {greeting}")

socket_path = os.path.join(os.path.dirname(__file__), "socket")
start_server = websockets.unix_serve(hello, socket_path)

asyncio.get_event_loop().run_until_complete(start_server)
asyncio.get_event_loop().run_forever()
