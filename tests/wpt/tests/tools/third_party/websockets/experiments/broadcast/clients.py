#!/usr/bin/env python

import asyncio
import statistics
import sys
import time

import websockets


LATENCIES = {}


async def log_latency(interval):
    while True:
        await asyncio.sleep(interval)
        p = statistics.quantiles(LATENCIES.values(), n=100)
        print(f"clients = {len(LATENCIES)}")
        print(
            f"p50 = {p[49] / 1e6:.1f}ms, "
            f"p95 = {p[94] / 1e6:.1f}ms, "
            f"p99 = {p[98] / 1e6:.1f}ms"
        )
        print()


async def client():
    try:
        async with websockets.connect(
            "ws://localhost:8765",
            ping_timeout=None,
        ) as websocket:
            async for msg in websocket:
                client_time = time.time_ns()
                server_time = int(msg[:19].decode())
                LATENCIES[websocket] = client_time - server_time
    except Exception as exc:
        print(exc)


async def main(count, interval):
    asyncio.create_task(log_latency(interval))
    clients = []
    for _ in range(count):
        clients.append(asyncio.create_task(client()))
        await asyncio.sleep(0.001)  # 1ms between each connection
    await asyncio.wait(clients)


if __name__ == "__main__":
    try:
        count = int(sys.argv[1])
        interval = float(sys.argv[2])
    except Exception as exc:
        print(f"Usage: {sys.argv[0]} count interval")
        print("    Connect <count> clients e.g. 1000")
        print("    Report latency every <interval> seconds e.g. 1")
        print()
        print(exc)
    else:
        asyncio.run(main(count, interval))
