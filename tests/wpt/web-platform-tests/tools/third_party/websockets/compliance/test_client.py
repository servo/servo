import json
import logging
import urllib.parse

import asyncio
import websockets


logging.basicConfig(level=logging.WARNING)

# Uncomment this line to make only websockets more verbose.
# logging.getLogger('websockets').setLevel(logging.DEBUG)


SERVER = "ws://127.0.0.1:8642"
AGENT = "websockets"


async def get_case_count(server):
    uri = f"{server}/getCaseCount"
    async with websockets.connect(uri) as ws:
        msg = ws.recv()
    return json.loads(msg)


async def run_case(server, case, agent):
    uri = f"{server}/runCase?case={case}&agent={agent}"
    async with websockets.connect(uri, max_size=2 ** 25, max_queue=1) as ws:
        async for msg in ws:
            await ws.send(msg)


async def update_reports(server, agent):
    uri = f"{server}/updateReports?agent={agent}"
    async with websockets.connect(uri):
        pass


async def run_tests(server, agent):
    cases = await get_case_count(server)
    for case in range(1, cases + 1):
        print(f"Running test case {case} out of {cases}", end="\r")
        await run_case(server, case, agent)
    print(f"Ran {cases} test cases               ")
    await update_reports(server, agent)


main = run_tests(SERVER, urllib.parse.quote(AGENT))
asyncio.get_event_loop().run_until_complete(main)
