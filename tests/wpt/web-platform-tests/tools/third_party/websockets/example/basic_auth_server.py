#!/usr/bin/env python

# Server example with HTTP Basic Authentication over TLS

import asyncio
import websockets

async def hello(websocket, path):
    greeting = f"Hello {websocket.username}!"
    await websocket.send(greeting)

start_server = websockets.serve(
    hello, "localhost", 8765,
    create_protocol=websockets.basic_auth_protocol_factory(
        realm="example", credentials=("mary", "p@ssw0rd")
    ),
)

asyncio.get_event_loop().run_until_complete(start_server)
asyncio.get_event_loop().run_forever()
