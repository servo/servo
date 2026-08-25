#!/usr/bin/env python

import asyncio
import os
import signal

import websockets


async def echo(websocket):
    async for message in websocket:
        await websocket.send(message)


async def main():
    # Set the stop condition when receiving SIGTERM.
    loop = asyncio.get_running_loop()
    stop = loop.create_future()
    loop.add_signal_handler(signal.SIGTERM, stop.set_result, None)

    async with websockets.unix_serve(
        echo,
        path=f"{os.environ['SUPERVISOR_PROCESS_NAME']}.sock",
    ):
        await stop


if __name__ == "__main__":
    asyncio.run(main())
