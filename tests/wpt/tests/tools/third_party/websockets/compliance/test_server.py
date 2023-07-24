import logging

import asyncio
import websockets


logging.basicConfig(level=logging.WARNING)

# Uncomment this line to make only websockets more verbose.
# logging.getLogger('websockets').setLevel(logging.DEBUG)


HOST, PORT = "127.0.0.1", 8642


async def echo(ws, path):
    async for msg in ws:
        await ws.send(msg)


start_server = websockets.serve(echo, HOST, PORT, max_size=2 ** 25, max_queue=1)

try:
    asyncio.get_event_loop().run_until_complete(start_server)
    asyncio.get_event_loop().run_forever()
except KeyboardInterrupt:
    pass
