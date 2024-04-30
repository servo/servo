#!/usr/bin/env python

import asyncio
import functools
import os
import sys
import time

import websockets


CLIENTS = set()


async def send(websocket, message):
    try:
        await websocket.send(message)
    except websockets.ConnectionClosed:
        pass


async def relay(queue, websocket):
    while True:
        message = await queue.get()
        await websocket.send(message)


class PubSub:
    def __init__(self):
        self.waiter = asyncio.Future()

    def publish(self, value):
        waiter, self.waiter = self.waiter, asyncio.Future()
        waiter.set_result((value, self.waiter))

    async def subscribe(self):
        waiter = self.waiter
        while True:
            value, waiter = await waiter
            yield value

    __aiter__ = subscribe


PUBSUB = PubSub()


async def handler(websocket, method=None):
    if method in ["default", "naive", "task", "wait"]:
        CLIENTS.add(websocket)
        try:
            await websocket.wait_closed()
        finally:
            CLIENTS.remove(websocket)
    elif method == "queue":
        queue = asyncio.Queue()
        relay_task = asyncio.create_task(relay(queue, websocket))
        CLIENTS.add(queue)
        try:
            await websocket.wait_closed()
        finally:
            CLIENTS.remove(queue)
            relay_task.cancel()
    elif method == "pubsub":
        async for message in PUBSUB:
            await websocket.send(message)
    else:
        raise NotImplementedError(f"unsupported method: {method}")


async def broadcast(method, size, delay):
    """Broadcast messages at regular intervals."""
    load_average = 0
    time_average = 0
    pc1, pt1 = time.perf_counter_ns(), time.process_time_ns()
    await asyncio.sleep(delay)
    while True:
        print(f"clients = {len(CLIENTS)}")
        pc0, pt0 = time.perf_counter_ns(), time.process_time_ns()
        load_average = 0.9 * load_average + 0.1 * (pt0 - pt1) / (pc0 - pc1)
        print(
            f"load = {(pt0 - pt1) / (pc0 - pc1) * 100:.1f}% / "
            f"average = {load_average * 100:.1f}%, "
            f"late = {(pc0 - pc1 - delay * 1e9) / 1e6:.1f} ms"
        )
        pc1, pt1 = pc0, pt0

        assert size > 20
        message = str(time.time_ns()).encode() + b" " + os.urandom(size - 20)

        if method == "default":
            websockets.broadcast(CLIENTS, message)
        elif method == "naive":
            # Since the loop can yield control, make a copy of CLIENTS
            # to avoid: RuntimeError: Set changed size during iteration
            for websocket in CLIENTS.copy():
                await send(websocket, message)
        elif method == "task":
            for websocket in CLIENTS:
                asyncio.create_task(send(websocket, message))
        elif method == "wait":
            if CLIENTS:  # asyncio.wait doesn't accept an empty list
                await asyncio.wait(
                    [
                        asyncio.create_task(send(websocket, message))
                        for websocket in CLIENTS
                    ]
                )
        elif method == "queue":
            for queue in CLIENTS:
                queue.put_nowait(message)
        elif method == "pubsub":
            PUBSUB.publish(message)
        else:
            raise NotImplementedError(f"unsupported method: {method}")

        pc2 = time.perf_counter_ns()
        wait = delay + (pc1 - pc2) / 1e9
        time_average = 0.9 * time_average + 0.1 * (pc2 - pc1)
        print(
            f"broadcast = {(pc2 - pc1) / 1e6:.1f}ms / "
            f"average = {time_average / 1e6:.1f}ms, "
            f"wait = {wait * 1e3:.1f}ms"
        )
        await asyncio.sleep(wait)
        print()


async def main(method, size, delay):
    async with websockets.serve(
        functools.partial(handler, method=method),
        "localhost",
        8765,
        compression=None,
        ping_timeout=None,
    ):
        await broadcast(method, size, delay)


if __name__ == "__main__":
    try:
        method = sys.argv[1]
        assert method in ["default", "naive", "task", "wait", "queue", "pubsub"]
        size = int(sys.argv[2])
        delay = float(sys.argv[3])
    except Exception as exc:
        print(f"Usage: {sys.argv[0]} method size delay")
        print("    Start a server broadcasting messages with <method> e.g. naive")
        print("    Send a payload of <size> bytes every <delay> seconds")
        print()
        print(exc)
    else:
        asyncio.run(main(method, size, delay))
