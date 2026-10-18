# mypy: allow-untyped-defs

"""Connects one WebSocket-over-H3 stream to a pywebsocket3 handler.

aioquic gives this server WebSocket data as HTTP/3 DATA events. pywebsocket3
expects to read and write data through a socket-like object. This file provides
the small request, connection, and session objects that translate between those
two APIs.
"""

import asyncio
import logging
import threading
from typing import Dict, List, Optional, Tuple, TYPE_CHECKING, Union

from pywebsocket3 import dispatch

from .headers import H3Headers

if TYPE_CHECKING:
    from .websocket_h3_server import WebSocketH3Protocol


_logger: logging.Logger = logging.getLogger(__name__)

DEFAULT_MAX_UNREAD_WEBSOCKET_BYTES = 16 * 1024 * 1024


_HeaderName = Union[str, bytes]
_HeaderValue = Union[str, bytes]


class _WebSocketH3Input:
    """Bounded, blocking reader for the synchronous pywebsocket handler."""

    def __init__(
        self, max_unread_websocket_bytes: int = DEFAULT_MAX_UNREAD_WEBSOCKET_BYTES
    ) -> None:
        self._condition = threading.Condition()
        self._buffer = bytearray()
        self._closed = False
        self._max_unread_websocket_bytes = max_unread_websocket_bytes

    def feed_data(self, data: bytes) -> bool:
        with self._condition:
            if (self._closed or len(self._buffer) + len(data) >
                    self._max_unread_websocket_bytes):
                return False
            self._buffer.extend(data)
            self._condition.notify()
            return True

    def read(self, length: int) -> bytes:
        with self._condition:
            while len(self._buffer) < length and not self._closed:
                self._condition.wait()
            count = min(length, len(self._buffer))
            data = bytes(self._buffer[:count])
            del self._buffer[:count]
            return data

    def close(self, discard: bool = False) -> None:
        with self._condition:
            self._closed = True
            if discard:
                self._buffer.clear()
            self._condition.notify_all()


class _WebSocketH3Connection:
    """Connection object passed to pywebsocket3.

    pywebsocket3 expects request.connection to look like a mod_python
    connection. This object provides the socket-like fields it reads while the
    WebSocket/H3 stream is handled.
    """

    def __init__(
        self,
        protocol: "WebSocketH3Protocol",
        stream_id: int,
        rfile: _WebSocketH3Input,
        loop: asyncio.AbstractEventLoop,
    ) -> None:
        self._protocol = protocol
        self._stream_id = stream_id
        self._rfile = rfile
        self._loop = loop
        self.remote_addr = ("unknown", 0)
        self._handshake_data: Optional[list[bytes]] = []
        self._handshake_size = 0

    def read(self, length: int) -> bytes:
        return self._rfile.read(length)

    def write(self, data: bytes) -> None:
        if self._handshake_data is not None:
            if self._handshake_size + len(data) > 1024 * 1024:
                raise OSError("WebSocket handshake output exceeds 1 MiB")
            self._handshake_data.append(data)
            self._handshake_size += len(data)
            return
        write = self._async_write(data)
        try:
            future = asyncio.run_coroutine_threadsafe(write, self._loop)
        except RuntimeError:
            write.close()
            raise
        future.result(timeout=30)

    def finish_handshake(self) -> None:
        data = self._handshake_data
        assert data is not None
        self._handshake_data = None
        assert self._protocol._http is not None
        for chunk in data:
            self._protocol._http.send_data(
                stream_id=self._stream_id, data=chunk, end_stream=False)
        if data:
            self._protocol.transmit()

    async def _async_write(self, data: bytes) -> None:
        if self._protocol._http is None:
            return
        self._protocol._http.send_data(
            stream_id=self._stream_id, data=data, end_stream=False)
        self._protocol.transmit()


class _WebSocketH3ResponseHeaders(Dict[_HeaderName, _HeaderValue]):
    """Mutable response headers populated by the H3 handshaker."""

    pass


class _WebSocketH3Request:
    """Request object passed to pywebsocket3.

    pywebsocket3 expects a mod_python-style request. This object provides the
    fields it reads during the WebSocket handshake, using values from the
    HTTP/3 CONNECT request.
    """

    def __init__(
        self,
        headers: H3Headers,
        connection: _WebSocketH3Connection,
    ) -> None:
        self.connection = connection
        self.protocol = "HTTP/3"
        self.uri = headers.get("path", "/")
        self.unparsed_uri = self.uri
        self.method = headers.get("method", "")
        self.headers_in = headers
        self.headers_out = _WebSocketH3ResponseHeaders()
        self.extra_headers: List[Tuple[_HeaderName, _HeaderValue]] = []
        self._dispatcher: Optional[dispatch.Dispatcher] = None
        self._status = 0

    @property
    def status(self) -> int:
        return self._status

    @status.setter
    def status(self, value: int) -> None:
        self._status = value

    def is_https(self) -> bool:
        return True


class _WebSocketH3Session:
    """Bridge one H3 CONNECT stream to pywebsocket3's blocking API."""

    def __init__(
        self,
        protocol: "WebSocketH3Protocol",
        stream_id: int,
        headers: H3Headers,
        dispatcher: dispatch.Dispatcher,
        loop: asyncio.AbstractEventLoop,
        logger: Optional[logging.Logger] = None,
        max_unread_websocket_bytes: int = DEFAULT_MAX_UNREAD_WEBSOCKET_BYTES,
    ) -> None:
        self.stream_id = stream_id
        self._protocol = protocol
        self._dispatcher = dispatcher
        self._loop = loop
        self._logger = logger if logger is not None else _logger
        self._handler_thread: Optional[threading.Thread] = None
        self._rfile = _WebSocketH3Input(max_unread_websocket_bytes)
        connection = _WebSocketH3Connection(
            protocol, stream_id, self._rfile, loop)
        self._request = _WebSocketH3Request(headers, connection)

    @property
    def request(self) -> _WebSocketH3Request:
        return self._request

    def start(self) -> None:
        self._handler_thread = threading.Thread(
            target=self._run_handler,
            name=f"ws-h3-handler-{self.stream_id}",
            daemon=True)
        self._handler_thread.start()

    def feed_data(self, data: bytes) -> bool:
        return self._rfile.feed_data(data)

    def close(self) -> None:
        self._rfile.close()

    def abort(self) -> None:
        self._rfile.close(discard=True)

    def _run_handler(self) -> None:
        try:
            self._dispatcher.transfer_data(self._request)  # type: ignore
        except Exception:
            self._logger.exception(
                "WebSocket/H3 handler failed for stream %d", self.stream_id)

        try:
            end_stream = self._async_end_stream()
            try:
                future = asyncio.run_coroutine_threadsafe(
                    end_stream, self._loop)
            except RuntimeError:
                end_stream.close()
                raise
            future.result(timeout=5)
        except Exception:
            self._logger.debug("WebSocket/H3 stream %d was already closed",
                               self.stream_id)
        finally:
            self.close()
            self._rfile.close()
            try:
                self._loop.call_soon_threadsafe(self._remove_from_protocol)
            except RuntimeError:
                pass

    async def _async_end_stream(self) -> None:
        if self._protocol._http is None:
            return
        self._protocol._http.send_data(
            stream_id=self.stream_id, data=b"", end_stream=True)
        self._protocol.transmit()

    def _remove_from_protocol(self) -> None:
        if self._protocol._sessions.get(self.stream_id) is self:
            del self._protocol._sessions[self.stream_id]
