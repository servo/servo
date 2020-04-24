import argparse
import asyncio
import importlib
import json
import logging
import os
import time
from collections import deque
from email.utils import formatdate
from typing import Callable, Deque, Dict, List, Optional, Union, cast

import wsproto
import wsproto.events

import aioquic
from aioquic.asyncio import QuicConnectionProtocol, serve
from aioquic.h0.connection import H0_ALPN, H0Connection
from aioquic.h3.connection import H3_ALPN, H3Connection
from aioquic.h3.events import DataReceived, H3Event, HeadersReceived
from aioquic.h3.exceptions import NoAvailablePushIDError
from aioquic.quic.configuration import QuicConfiguration
from aioquic.quic.events import DatagramFrameReceived, ProtocolNegotiated, QuicEvent
from aioquic.quic.logger import QuicLogger, QuicLoggerTrace
from aioquic.tls import SessionTicket

try:
    import uvloop
except ImportError:
    uvloop = None

AsgiApplication = Callable
HttpConnection = Union[H0Connection, H3Connection]

SERVER_NAME = "aioquic/" + aioquic.__version__


class HttpRequestHandler:
    def __init__(
        self,
        *,
        authority: bytes,
        connection: HttpConnection,
        protocol: QuicConnectionProtocol,
        scope: Dict,
        stream_ended: bool,
        stream_id: int,
        transmit: Callable[[], None],
    ) -> None:
        self.authority = authority
        self.connection = connection
        self.protocol = protocol
        self.queue: asyncio.Queue[Dict] = asyncio.Queue()
        self.scope = scope
        self.stream_id = stream_id
        self.transmit = transmit

        if stream_ended:
            self.queue.put_nowait({"type": "http.request"})

    def http_event_received(self, event: H3Event) -> None:
        if isinstance(event, DataReceived):
            self.queue.put_nowait(
                {
                    "type": "http.request",
                    "body": event.data,
                    "more_body": not event.stream_ended,
                }
            )
        elif isinstance(event, HeadersReceived) and event.stream_ended:
            self.queue.put_nowait(
                {"type": "http.request", "body": b"", "more_body": False}
            )

    async def run_asgi(self, app: AsgiApplication) -> None:
        await application(self.scope, self.receive, self.send)

    async def receive(self) -> Dict:
        return await self.queue.get()

    async def send(self, message: Dict) -> None:
        if message["type"] == "http.response.start":
            self.connection.send_headers(
                stream_id=self.stream_id,
                headers=[
                    (b":status", str(message["status"]).encode()),
                    (b"server", SERVER_NAME.encode()),
                    (b"date", formatdate(time.time(), usegmt=True).encode()),
                ]
                + [(k, v) for k, v in message["headers"]],
            )
        elif message["type"] == "http.response.body":
            self.connection.send_data(
                stream_id=self.stream_id,
                data=message.get("body", b""),
                end_stream=not message.get("more_body", False),
            )
        elif message["type"] == "http.response.push" and isinstance(
            self.connection, H3Connection
        ):
            request_headers = [
                (b":method", b"GET"),
                (b":scheme", b"https"),
                (b":authority", self.authority),
                (b":path", message["path"].encode()),
            ] + [(k, v) for k, v in message["headers"]]

            # send push promise
            try:
                push_stream_id = self.connection.send_push_promise(
                    stream_id=self.stream_id, headers=request_headers
                )
            except NoAvailablePushIDError:
                return

            # fake request
            cast(HttpServerProtocol, self.protocol).http_event_received(
                HeadersReceived(
                    headers=request_headers, stream_ended=True, stream_id=push_stream_id
                )
            )
        self.transmit()


class WebSocketHandler:
    def __init__(
        self,
        *,
        connection: HttpConnection,
        scope: Dict,
        stream_id: int,
        transmit: Callable[[], None],
    ) -> None:
        self.closed = False
        self.connection = connection
        self.http_event_queue: Deque[DataReceived] = deque()
        self.queue: asyncio.Queue[Dict] = asyncio.Queue()
        self.scope = scope
        self.stream_id = stream_id
        self.transmit = transmit
        self.websocket: Optional[wsproto.Connection] = None

    def http_event_received(self, event: H3Event) -> None:
        if isinstance(event, DataReceived) and not self.closed:
            if self.websocket is not None:
                self.websocket.receive_data(event.data)

                for ws_event in self.websocket.events():
                    self.websocket_event_received(ws_event)
            else:
                # delay event processing until we get `websocket.accept`
                # from the ASGI application
                self.http_event_queue.append(event)

    def websocket_event_received(self, event: wsproto.events.Event) -> None:
        if isinstance(event, wsproto.events.TextMessage):
            self.queue.put_nowait({"type": "websocket.receive", "text": event.data})
        elif isinstance(event, wsproto.events.Message):
            self.queue.put_nowait({"type": "websocket.receive", "bytes": event.data})
        elif isinstance(event, wsproto.events.CloseConnection):
            self.queue.put_nowait({"type": "websocket.disconnect", "code": event.code})

    async def run_asgi(self, app: AsgiApplication) -> None:
        self.queue.put_nowait({"type": "websocket.connect"})

        try:
            await application(self.scope, self.receive, self.send)
        finally:
            if not self.closed:
                await self.send({"type": "websocket.close", "code": 1000})

    async def receive(self) -> Dict:
        return await self.queue.get()

    async def send(self, message: Dict) -> None:
        data = b""
        end_stream = False
        if message["type"] == "websocket.accept":
            subprotocol = message.get("subprotocol")

            self.websocket = wsproto.Connection(wsproto.ConnectionType.SERVER)

            headers = [
                (b":status", b"200"),
                (b"server", SERVER_NAME.encode()),
                (b"date", formatdate(time.time(), usegmt=True).encode()),
            ]
            if subprotocol is not None:
                headers.append((b"sec-websocket-protocol", subprotocol.encode()))
            self.connection.send_headers(stream_id=self.stream_id, headers=headers)

            # consume backlog
            while self.http_event_queue:
                self.http_event_received(self.http_event_queue.popleft())

        elif message["type"] == "websocket.close":
            if self.websocket is not None:
                data = self.websocket.send(
                    wsproto.events.CloseConnection(code=message["code"])
                )
            else:
                self.connection.send_headers(
                    stream_id=self.stream_id, headers=[(b":status", b"403")]
                )
            end_stream = True
        elif message["type"] == "websocket.send":
            if message.get("text") is not None:
                data = self.websocket.send(
                    wsproto.events.TextMessage(data=message["text"])
                )
            elif message.get("bytes") is not None:
                data = self.websocket.send(
                    wsproto.events.Message(data=message["bytes"])
                )

        if data:
            self.connection.send_data(
                stream_id=self.stream_id, data=data, end_stream=end_stream
            )
        if end_stream:
            self.closed = True
        self.transmit()


Handler = Union[HttpRequestHandler, WebSocketHandler]


class HttpServerProtocol(QuicConnectionProtocol):
    def __init__(self, *args, **kwargs) -> None:
        super().__init__(*args, **kwargs)
        self._handlers: Dict[int, Handler] = {}
        self._http: Optional[HttpConnection] = None

    def http_event_received(self, event: H3Event) -> None:
        if isinstance(event, HeadersReceived) and event.stream_id not in self._handlers:
            authority = None
            headers = []
            http_version = "0.9" if isinstance(self._http, H0Connection) else "3"
            raw_path = b""
            method = ""
            protocol = None
            for header, value in event.headers:
                if header == b":authority":
                    authority = value
                    headers.append((b"host", value))
                elif header == b":method":
                    method = value.decode()
                elif header == b":path":
                    raw_path = value
                elif header == b":protocol":
                    protocol = value.decode()
                elif header and not header.startswith(b":"):
                    headers.append((header, value))

            if b"?" in raw_path:
                path_bytes, query_string = raw_path.split(b"?", maxsplit=1)
            else:
                path_bytes, query_string = raw_path, b""
            path = path_bytes.decode()

            # FIXME: add a public API to retrieve peer address
            client_addr = self._http._quic._network_paths[0].addr
            client = (client_addr[0], client_addr[1])

            handler: Handler
            scope: Dict
            if method == "CONNECT" and protocol == "websocket":
                subprotocols: List[str] = []
                for header, value in event.headers:
                    if header == b"sec-websocket-protocol":
                        subprotocols = [x.strip() for x in value.decode().split(",")]
                scope = {
                    "client": client,
                    "headers": headers,
                    "http_version": http_version,
                    "method": method,
                    "path": path,
                    "query_string": query_string,
                    "raw_path": raw_path,
                    "root_path": "",
                    "scheme": "wss",
                    "subprotocols": subprotocols,
                    "type": "websocket",
                }
                handler = WebSocketHandler(
                    connection=self._http,
                    scope=scope,
                    stream_id=event.stream_id,
                    transmit=self.transmit,
                )
            else:
                extensions: Dict[str, Dict] = {}
                if isinstance(self._http, H3Connection):
                    extensions["http.response.push"] = {}
                scope = {
                    "client": client,
                    "extensions": extensions,
                    "headers": headers,
                    "http_version": http_version,
                    "method": method,
                    "path": path,
                    "query_string": query_string,
                    "raw_path": raw_path,
                    "root_path": "",
                    "scheme": "https",
                    "type": "http",
                }
                handler = HttpRequestHandler(
                    authority=authority,
                    connection=self._http,
                    protocol=self,
                    scope=scope,
                    stream_ended=event.stream_ended,
                    stream_id=event.stream_id,
                    transmit=self.transmit,
                )
            self._handlers[event.stream_id] = handler
            asyncio.ensure_future(handler.run_asgi(application))
        elif (
            isinstance(event, (DataReceived, HeadersReceived))
            and event.stream_id in self._handlers
        ):
            handler = self._handlers[event.stream_id]
            handler.http_event_received(event)

    def quic_event_received(self, event: QuicEvent) -> None:
        if isinstance(event, ProtocolNegotiated):
            if event.alpn_protocol.startswith("h3-"):
                self._http = H3Connection(self._quic)
            elif event.alpn_protocol.startswith("hq-"):
                self._http = H0Connection(self._quic)
        elif isinstance(event, DatagramFrameReceived):
            if event.data == b"quack":
                self._quic.send_datagram_frame(b"quack-ack")

        #  pass event to the HTTP layer
        if self._http is not None:
            for http_event in self._http.handle_event(event):
                self.http_event_received(http_event)


class SessionTicketStore:
    """
    Simple in-memory store for session tickets.
    """

    def __init__(self) -> None:
        self.tickets: Dict[bytes, SessionTicket] = {}

    def add(self, ticket: SessionTicket) -> None:
        self.tickets[ticket.ticket] = ticket

    def pop(self, label: bytes) -> Optional[SessionTicket]:
        return self.tickets.pop(label, None)


class QuicLoggerCustom(QuicLogger):
    """
    Custom QUIC logger which writes one trace per file.
    """

    def __init__(self, path: str) -> None:
        if not os.path.isdir(path):
            raise ValueError("QUIC log output directory '%s' does not exist" % path)
        self.path = path
        super().__init__()

    def end_trace(self, trace: QuicLoggerTrace) -> None:
        trace_dict = trace.to_dict()
        trace_path = os.path.join(
            self.path, trace_dict["common_fields"]["ODCID"] + ".qlog"
        )
        with open(trace_path, "w") as logger_fp:
            json.dump({"qlog_version": "draft-01", "traces": [trace_dict]}, logger_fp)
        self._traces.remove(trace)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="QUIC server")
    parser.add_argument(
        "app",
        type=str,
        nargs="?",
        default="demo:app",
        help="the ASGI application as <module>:<attribute>",
    )
    parser.add_argument(
        "-c",
        "--certificate",
        type=str,
        required=True,
        help="load the TLS certificate from the specified file",
    )
    parser.add_argument(
        "--host",
        type=str,
        default="::",
        help="listen on the specified address (defaults to ::)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=4433,
        help="listen on the specified port (defaults to 4433)",
    )
    parser.add_argument(
        "-k",
        "--private-key",
        type=str,
        required=True,
        help="load the TLS private key from the specified file",
    )
    parser.add_argument(
        "-l",
        "--secrets-log",
        type=str,
        help="log secrets to a file, for use with Wireshark",
    )
    parser.add_argument(
        "-q", "--quic-log", type=str, help="log QUIC events to a file in QLOG format"
    )
    parser.add_argument(
        "-r",
        "--stateless-retry",
        action="store_true",
        help="send a stateless retry for new connections",
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true", help="increase logging verbosity"
    )
    args = parser.parse_args()

    logging.basicConfig(
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
        level=logging.DEBUG if args.verbose else logging.INFO,
    )

    # import ASGI application
    module_str, attr_str = args.app.split(":", maxsplit=1)
    module = importlib.import_module(module_str)
    application = getattr(module, attr_str)

    # create QUIC logger
    if args.quic_log:
        quic_logger = QuicLoggerCustom(args.quic_log)
    else:
        quic_logger = None

    # open SSL log file
    if args.secrets_log:
        secrets_log_file = open(args.secrets_log, "a")
    else:
        secrets_log_file = None

    configuration = QuicConfiguration(
        alpn_protocols=H3_ALPN + H0_ALPN + ["siduck"],
        is_client=False,
        max_datagram_frame_size=65536,
        quic_logger=quic_logger,
        secrets_log_file=secrets_log_file,
    )

    # load SSL certificate and key
    configuration.load_cert_chain(args.certificate, args.private_key)

    ticket_store = SessionTicketStore()

    if uvloop is not None:
        uvloop.install()
    loop = asyncio.get_event_loop()
    loop.run_until_complete(
        serve(
            args.host,
            args.port,
            configuration=configuration,
            create_protocol=HttpServerProtocol,
            session_ticket_fetcher=ticket_store.pop,
            session_ticket_handler=ticket_store.add,
            stateless_retry=args.stateless_retry,
        )
    )
    try:
        loop.run_forever()
    except KeyboardInterrupt:
        pass
