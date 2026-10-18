# mypy: allow-untyped-defs

"""HTTP/3 adapters for serving WPT resources through wptserve.

The WebSocket H3 server must serve more than WebSocket CONNECT streams: tests
also load HTML, JavaScript, CSS, Python handlers, and .sub. resources from the
same H3 origin. wptserve already knows how to route those requests and apply
the WPT response pipeline, but it expects HTTP/1-style request handler objects.

This module provides the small compatibility layer between aioquic HTTP/3
events and wptserve. It builds request/response wrapper objects for H3 streams
and writes wptserve responses back as HTTP/3 HEADERS and DATA frames.
"""

import logging
from typing import Any, BinaryIO, Iterable, List, Optional, Tuple, Union

from wptserve.utils import isomorphic_encode

from .headers import H3Headers


_logger: logging.Logger = logging.getLogger(__name__)

_H3_FORBIDDEN_RESPONSE_HEADERS = frozenset((
    "connection",
    "keep-alive",
    "proxy-connection",
    "transfer-encoding",
    "upgrade",
))

_HeaderName = Union[str, bytes]
_HeaderValue = Union[str, bytes, int]
_HeaderList = Iterable[Tuple[_HeaderName, _HeaderValue]]


class _H3ServerAdapter:
    """Provides the server fields wptserve needs for an H3 request."""

    def __init__(
        self,
        router: Any,
        host: str,
        port: int,
        rewriter: Optional[Any] = None,
        config: Optional[Any] = None,
    ) -> None:
        self.router = router
        self.scheme = "https"
        self.server_address = (host, port)
        self.latency = None
        self.rewriter = rewriter if rewriter is not None else _NoOpRewriter()
        self.config = config


class _NoOpRewriter:
    """Default rewriter used when no wptserve rewrites are configured."""

    def rewrite(self, handler: Any) -> None:
        pass


class _H3RequestHandlerAdapter:
    """Provides the request handler fields wptserve needs for an H3 request."""

    def __init__(
        self,
        server: _H3ServerAdapter,
        headers: H3Headers,
        rfile: BinaryIO,
        stream_id: int,
        protocol: Any,
    ) -> None:
        self.server = server
        self.headers = headers
        self.command = headers.get("method", "GET")
        self.path = headers.get("path", "/")
        self.protocol_version = "HTTP/3"
        self.client_address = ("unknown", 0)
        self.raw_requestline = b""
        self.rfile = rfile
        self.h3_stream_id = stream_id
        self.h3_protocol = protocol


class H3ResponseWriter:
    """Write wptserve responses as HTTP/3 HEADERS and DATA frames."""

    def __init__(self, handler: Any, response: Any) -> None:
        self._protocol = handler.h3_protocol
        self._stream_id = handler.h3_stream_id
        self._response = response
        self._handler = handler
        self.stream_ended = False
        self.content_written = False
        self.request = response.request
        self.logger = getattr(self._protocol, "_logger", _logger)

    def encode(self, data: Union[str, bytes]) -> bytes:
        if isinstance(data, bytes):
            return data
        return data.encode(self._response.encoding)

    def _send(self, method: str, **kwargs: Any) -> None:
        def send() -> None:
            h3 = self._protocol._http
            if h3 is not None:
                getattr(h3, method)(**kwargs)
                self._protocol.transmit()

        if hasattr(self._protocol, "_call_on_loop"):
            self._protocol._call_on_loop(send)
        else:
            send()

    def write_headers(
        self,
        headers: _HeaderList,
        status_code: int,
        status_message: Optional[Any] = None,
        stream_id: Optional[int] = None,
        last: bool = False,
    ) -> None:
        stream_id = self._stream_id if stream_id is None else stream_id
        if self._protocol._http is None:
            return

        end_stream = last or self.request.method == "HEAD"
        formatted_headers: List[Tuple[bytes, bytes]] = [
            (b":status", str(status_code).encode())
        ]
        for key, value in headers:
            header_name = key.decode("ascii") if isinstance(key, bytes) else key
            header_value = (value if isinstance(value, bytes)
                            else isomorphic_encode(str(value)))
            header_name = header_name.lower()
            if header_name in _H3_FORBIDDEN_RESPONSE_HEADERS:
                continue
            formatted_headers.append((header_name.encode(), header_value))

        self._send("send_headers", stream_id=stream_id, headers=formatted_headers,
                   end_stream=end_stream)
        self.content_written = True
        self.stream_ended = end_stream

    def write_data(
        self,
        item: Any,
        last: bool = False,
        stream_id: Optional[int] = None,
    ) -> None:
        stream_id = self._stream_id if stream_id is None else stream_id
        if self._protocol._http is None:
            return

        data = self.encode(item) if isinstance(item, (str, bytes)) else item.read()
        if isinstance(data, str):
            data = self.encode(data)

        chunk_size = 65536
        if len(data) == 0:
            self._send("send_data", stream_id=stream_id, data=b"",
                       end_stream=last)
            self.content_written = True
            self.stream_ended = last
            return

        offset = 0
        while offset < len(data):
            chunk = data[offset:offset + chunk_size]
            offset += chunk_size
            end_stream = last and offset >= len(data)
            self._send("send_data", stream_id=stream_id, data=chunk,
                       end_stream=end_stream)

        self.content_written = True
        self.stream_ended = last

    def end_stream(self) -> None:
        if self.stream_ended:
            return
        if self._protocol._http is None:
            return
        self._send("send_data", stream_id=self._stream_id, data=b"",
                   end_stream=True)
        self.stream_ended = True


def create_h3_request_response_classes() -> Tuple[Any, Any]:
    from wptserve.request import Request
    from wptserve.response import Response

    class _H3Request(Request):
        """wptserve Request subclass carrying the H3 stream id."""

        def __init__(self, request_handler):
            self.h3_stream_id = request_handler.h3_stream_id
            self.frames = []
            super().__init__(request_handler)
            self.server.config = request_handler.server.config

    class _H3Response(Response):
        """wptserve Response subclass that writes through H3ResponseWriter."""

        def __init__(self, handler, request):
            super().__init__(handler, request,
                             response_writer_cls=H3ResponseWriter)

        def write_status_headers(self) -> None:
            self.writer.write_headers(self.headers, *self.status)

        def write_content(self):
            if self.request.method == "HEAD" and not self.send_body_for_head_request:
                return

            item = None
            item_iter = self.iter_content()
            try:
                item = next(item_iter)
                while True:
                    check_last = next(item_iter)
                    self.writer.write_data(item, last=False)
                    item = check_last
            except StopIteration:
                if item:
                    self.writer.write_data(item, last=True)

    return _H3Request, _H3Response
