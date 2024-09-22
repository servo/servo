import asyncio
import json
import logging
import sys
from typing import Any, Callable, Coroutine, List, Optional, Mapping

import websockets

from websockets.exceptions import ConnectionClosed

logger = logging.getLogger("webdriver.bidi")


def get_running_loop() -> asyncio.AbstractEventLoop:
    if sys.version_info >= (3, 7):
        return asyncio.get_running_loop()
    else:
        # Unlike the above, this will actually create an event loop
        # if there isn't one; hopefully running tests in Python >= 3.7
        # will allow us to catch any behaviour difference
        # (Needs to be in else for mypy to believe this is reachable)
        return asyncio.get_event_loop()


class Transport:
    """Low level message handler for the WebSockets connection"""
    def __init__(self, url: str,
                 msg_handler: Callable[[Mapping[str, Any]], Coroutine[Any, Any, None]],
                 loop: Optional[asyncio.AbstractEventLoop] = None):
        self.url = url
        self.connection: Optional[websockets.WebSocketClientProtocol] = None  # type: ignore
        self.msg_handler = msg_handler
        self.send_buf: List[Mapping[str, Any]] = []

        if loop is None:
            loop = get_running_loop()
        self.loop = loop

        self.read_message_task: Optional[asyncio.Task[Any]] = None

    async def start(self) -> None:
        # Default max_size of 1048576 bytes is too small for some messages.
        # 128MB should be enough.
        self.connection = await websockets.connect(self.url, max_size=128 * 1024 * 1024)  # type: ignore
        self.read_message_task = self.loop.create_task(self.read_messages())

        for msg in self.send_buf:
            await self._send(self.connection, msg)

    async def send(self, data: Mapping[str, Any]) -> None:
        if self.connection is not None:
            await self._send(self.connection, data)
        else:
            self.send_buf.append(data)

    @staticmethod
    async def _send(
        connection: websockets.WebSocketClientProtocol,  # type: ignore
        data: Mapping[str, Any]
    ) -> None:
        msg = json.dumps(data)
        logger.debug("→ %s", msg)
        await connection.send(msg)

    async def handle(self, msg: str) -> None:
        logger.debug("← %s", msg)
        data = json.loads(msg)
        await self.msg_handler(data)

    async def end(self) -> None:
        if self.connection:
            await self.connection.close()
            self.connection = None

    async def read_messages(self) -> None:
        assert self.connection is not None
        try:
            async for msg in self.connection:
                if not isinstance(msg, str):
                    raise ValueError("Got a binary message")
                await self.handle(msg)
        except ConnectionClosed:
            logger.debug("connection closed while reading messages")

    async def wait_closed(self) -> None:
        if self.connection and not self.connection.closed:
            await self.connection.wait_closed()
