#!/usr/bin/env python

# WS client example with HTTP Basic Authentication

import asyncio
import websockets

async def hello():
    uri = "ws://mary:p@ssw0rd@localhost:8765"
    async with websockets.connect(uri) as websocket:
        greeting = await websocket.recv()
        print(greeting)

asyncio.run(hello())
