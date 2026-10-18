# mypy: allow-untyped-defs

"""WebSocket over HTTP/3 server for WPT.

This is the main entry point for the WebSocket H3 server. It owns the QUIC
server lifecycle, creates the HTTP/3 connection object, dispatches incoming H3
events, handles Extended CONNECT requests for WebSocket streams, and serves
normal WPT resources over the same H3 port through wptserve adapters.
"""

import asyncio
import contextlib
import logging
import os
import ssl
import sys
import threading
from io import BytesIO
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple, cast

from aioquic.asyncio import QuicConnectionProtocol, serve
from aioquic.asyncio.client import connect
from aioquic.asyncio.protocol import QuicStreamAdapter
from aioquic.h3.connection import H3_ALPN, H3Connection
from aioquic.h3.events import DataReceived, H3Event, HeadersReceived
from aioquic.quic.configuration import QuicConfiguration
from aioquic.quic.connection import logger as quic_connection_logger
from aioquic.quic.connection import stream_is_client_initiated
from aioquic.quic.connection import stream_is_unidirectional
from aioquic.quic.events import (
    ConnectionTerminated,
    ProtocolNegotiated,
    QuicEvent,
    StreamReset,
)
from aioquic.tls import SessionTicket
from pywebsocket3 import dispatch
from pywebsocket3.handshake.base import AbortedByUserException
from pywebsocket3.handshake.base import HandshakeException

from tools import localpaths  # noqa: F401

from .h3_wptserve_adapter import (
    _H3_FORBIDDEN_RESPONSE_HEADERS,
    _H3RequestHandlerAdapter,
    _H3ServerAdapter,
    create_h3_request_response_classes,
)
from .headers import H3Headers
from .websocket_h3_session import (
    DEFAULT_MAX_UNREAD_WEBSOCKET_BYTES,
    _WebSocketH3Request,
    _WebSocketH3Session,
)
from .ws_h3_handshake import WsH3Handshaker


SERVER_NAME = "websocket-h3-server"
DEFAULT_MAX_PENDING_HANDSHAKES = 16
DEFAULT_MAX_ACTIVE_REQUESTS = 16

_logger: logging.Logger = logging.getLogger(__name__)

quic_connection_logger.setLevel(logging.WARNING)


class H3ConnectionForWebSocket(H3Connection):
    """H3Connection subclass that enables Extended CONNECT for WebSocket."""

    ENABLE_CONNECT_PROTOCOL = 0x08

    def _get_local_settings(self) -> Dict[int, int]:
        settings = super()._get_local_settings()
        settings[self.ENABLE_CONNECT_PROTOCOL] = 1
        return settings


class WebSocketH3Protocol(QuicConnectionProtocol):
    """Handles one client connection to the WebSocket-over-H3 server.

    One QUIC connection can carry many HTTP/3 streams. WebSocket CONNECT
    streams are passed to pywebsocket handlers. Normal GET and HEAD streams are
    served through the H3 wptserve adapter so test pages and resources can load
    from the same origin.
    """

    def __init__(
        self,
        *args: Any,
        ws_doc_root: str = "",
        router: Any = None,
        h3_server_adapter: Optional[_H3ServerAdapter] = None,
        h3_request_cls: Any = None,
        h3_response_cls: Any = None,
        logger: Optional[logging.Logger] = None,
        max_pending_handshakes: int = DEFAULT_MAX_PENDING_HANDSHAKES,
        max_active_requests: int = DEFAULT_MAX_ACTIVE_REQUESTS,
        max_unread_websocket_bytes: int = DEFAULT_MAX_UNREAD_WEBSOCKET_BYTES,
        **kwargs: Any,
    ) -> None:
        """Stores the state needed while handling one QUIC connection."""
        super().__init__(*args, **kwargs)
        self._http: Optional[H3ConnectionForWebSocket] = None
        self._sessions: Dict[int, _WebSocketH3Session] = {}
        self._active_requests = 0
        self._pending_handshakes = 0
        self._workers: Set[asyncio.Task[Any]] = set()
        self._queued_workers: Dict[int, asyncio.Task[Any]] = {}
        self._handshake_slots = asyncio.Semaphore(max_pending_handshakes)
        self._request_slots = asyncio.Semaphore(max_active_requests)
        self._max_unread_websocket_bytes = max_unread_websocket_bytes
        self._ws_doc_root = ws_doc_root
        self._router = router
        self._h3_server_adapter = h3_server_adapter
        self._h3_request_cls = h3_request_cls
        self._h3_response_cls = h3_response_cls
        self._logger = logger if logger is not None else _logger

    def quic_event_received(self, event: QuicEvent) -> None:
        """Receives raw QUIC events from aioquic and handles related sessions."""
        if isinstance(event, ProtocolNegotiated):
            self._http = H3ConnectionForWebSocket(self._quic)

        if self._http is not None:
            for h3_event in self._http.handle_event(event):
                self._h3_event_received(h3_event)

        if isinstance(event, ConnectionTerminated):
            self._close_all_sessions()
        elif isinstance(event, StreamReset):
            queued = self._queued_workers.pop(event.stream_id, None)
            if queued is not None:
                queued.cancel()
            session = self._sessions.pop(event.stream_id, None)
            if session is not None:
                session.abort()

    def _h3_event_received(self, event: H3Event) -> None:
        """Routes HTTP/3 events to the matching headers or data handler."""
        if isinstance(event, HeadersReceived):
            self._handle_headers(event)
        elif isinstance(event, DataReceived):
            self._handle_data(event)

    def _handle_headers(self, event: HeadersReceived) -> None:
        """Chooses how to handle a stream from its H3 request headers."""
        headers = dict(event.headers)
        method = headers.get(b":method")
        protocol = headers.get(b":protocol")

        if method == b"CONNECT" and protocol == b"websocket":
            self._handle_websocket_connect(event)
            if getattr(event, "stream_ended", False):
                self._handle_data(DataReceived(
                    data=b"", stream_id=event.stream_id, stream_ended=True))
        elif method == b"CONNECT":
            self._send_error(event.stream_id, 501)
        elif method in (b"GET", b"HEAD"):
            self._handle_request(event)
        else:
            self._send_error(event.stream_id, 405)

    def _handle_websocket_connect(self, event: HeadersReceived) -> None:
        """Handles a WebSocket CONNECT request on an HTTP/3 stream.

        This finds the matching pywebsocket handler, validates the WebSocket
        handshake, sends the H3 response headers, stores the session for later
        DATA frames, and starts the handler thread for the stream.
        """
        assert self._http is not None
        stream_id = event.stream_id
        h3_headers = H3Headers(event.headers)
        path = h3_headers.get("path", "/")
        ws_dispatcher = dispatch.Dispatcher(  # type: ignore
            self._ws_doc_root, None, False)

        loop = asyncio.get_running_loop()
        session = _WebSocketH3Session(
            self, stream_id, h3_headers, ws_dispatcher, loop, self._logger,
            max_unread_websocket_bytes=self._max_unread_websocket_bytes)
        self._sessions[stream_id] = session
        self._track_worker(self._finish_websocket_connect(
            session, path, ws_dispatcher), stream_id)

    def _track_worker(self, work: Any, stream_id: int) -> None:
        task = asyncio.create_task(work)
        self._workers.add(task)
        self._queued_workers[stream_id] = task
        task.add_done_callback(self._workers.discard)
        task.add_done_callback(
            lambda finished: self._queued_workers.pop(stream_id, None))

    async def _finish_websocket_connect(
        self, session: _WebSocketH3Session, path: str,
        ws_dispatcher: dispatch.Dispatcher,
    ) -> None:
        try:
            async with self._handshake_slots:
                self._queued_workers.pop(session.stream_id, None)
                if self._sessions.get(session.stream_id) is not session:
                    return
                self._pending_handshakes += 1
                try:
                    await self._do_websocket_connect(session, path, ws_dispatcher)
                finally:
                    self._pending_handshakes -= 1
        except asyncio.CancelledError:
            session.abort()
            if self._sessions.get(session.stream_id) is session:
                self._sessions.pop(session.stream_id)
            raise

    async def _do_websocket_connect(
        self, session: _WebSocketH3Session, path: str,
        ws_dispatcher: dispatch.Dispatcher,
    ) -> None:
        stream_id = session.stream_id
        request = session.request

        try:
            def handshake() -> bool:
                if not ws_dispatcher.get_handler_suite(path):  # type: ignore
                    return False
                WsH3Handshaker(request, ws_dispatcher).do_handshake()
                return True

            if not await asyncio.to_thread(handshake):
                if self._sessions.get(stream_id) is not session:
                    session.abort()
                    return
                self._logger.warning("No WebSocket handler found for %s", path)
                session.abort()
                self._sessions.pop(stream_id, None)
                self._send_error(stream_id, 404)
                return
        except asyncio.CancelledError:
            session.abort()
            self._sessions.pop(stream_id, None)
            raise
        except HandshakeException as error:
            status = error.status if error.status else 400
            self._logger.info(
                "WebSocket/H3 handshake failed for stream %d: %s",
                stream_id, error)
            session.abort()
            self._sessions.pop(stream_id, None)
            self._send_error(stream_id, status)
            return
        except AbortedByUserException:
            self._logger.info("WebSocket/H3 handshake aborted for stream %d",
                              stream_id)
            self._send_websocket_response(stream_id, request, end_stream=True)
            session.abort()
            self._sessions.pop(stream_id, None)
            return
        except Exception:
            self._logger.exception("WebSocket/H3 handshake failed for stream %d",
                                   stream_id)
            session.abort()
            self._sessions.pop(stream_id, None)
            self._send_error(stream_id, 500)
            return

        if self._sessions.get(stream_id) is not session:
            session.abort()
            return
        request._dispatcher = ws_dispatcher
        self._send_websocket_response(stream_id, request, end_stream=False)
        request.connection.finish_handshake()
        session.start()

    def _handle_request(self, event: HeadersReceived) -> None:
        """Serves a regular HTTP/3 request through wptserve.

        This handles non-WebSocket streams such as HTML, JavaScript, CSS,
        `.sub.` resources, and Python handlers. It adapts the H3 request to the
        request and response objects expected by wptserve, then lets the WPT
        router run the matching handler.
        """
        assert self._http is not None

        if self._router is None or self._h3_server_adapter is None:
            self._send_error(event.stream_id, 404)
            return
        self._track_worker(self._serve_request(event), event.stream_id)

    async def _serve_request(self, event: HeadersReceived) -> None:
        async with self._request_slots:
            self._queued_workers.pop(event.stream_id, None)
            self._active_requests += 1
            try:
                await asyncio.to_thread(self._serve_request_sync, event)
            finally:
                self._active_requests -= 1

    def _serve_request_sync(self, event: HeadersReceived) -> None:
        assert self._h3_server_adapter is not None
        h3_headers = H3Headers(event.headers)
        authority = h3_headers.get("authority")
        if authority and "Host" not in h3_headers:
            h3_headers["Host"] = authority
        request_handler = _H3RequestHandlerAdapter(
            server=self._h3_server_adapter,
            headers=h3_headers,
            rfile=BytesIO(b""),
            stream_id=event.stream_id,
            protocol=self,
        )

        try:
            request_handler.server.rewriter.rewrite(request_handler)
            with self._h3_request_cls(request_handler) as request:
                response = self._h3_response_cls(request_handler, request)
                handler = self._router.get_handler(request)

                if handler is None:
                    response.set_error(404)
                else:
                    if hasattr(handler, "base_path") and handler.base_path:
                        request.doc_root = handler.base_path
                    if hasattr(handler, "url_base") and handler.url_base != "/":
                        request.url_base = handler.url_base

                    try:
                        handler(request, response)
                    except Exception as error:
                        from wptserve.utils import HTTPException, get_error_cause
                        if isinstance(error, HTTPException):
                            exc = (get_error_cause(error)
                                   if 500 <= error.code < 600 else error)
                            response.set_error(error.code, exc)
                        else:
                            response.set_error(500, error)

                if not response.writer.content_written:
                    response.write()
                if not response.writer.stream_ended:
                    response.writer.end_stream()
        except Exception:
            self._logger.exception("Failed to serve HTTP/3 stream %d",
                                   event.stream_id)
            if self._http is not None:
                try:
                    self._call_on_loop(
                        self._send_error, event.stream_id, 500)
                except Exception:
                    self._logger.exception(
                        "Failed to send HTTP/3 error response for stream %d",
                        event.stream_id)

    def _call_on_loop(self, callback: Any, *args: Any) -> Any:
        async def call() -> Any:
            return callback(*args)

        action = call()
        try:
            future = asyncio.run_coroutine_threadsafe(action, self._loop)
        except RuntimeError:
            action.close()
            raise
        return future.result(timeout=30)

    def _handle_data(self, event: DataReceived) -> None:
        """Forwards H3 DATA frames to the matching WebSocket session."""
        session = self._sessions.get(event.stream_id)
        if session is None:
            return

        if event.data:
            if session.feed_data(event.data) is False:
                self._sessions.pop(event.stream_id, None)
                session.abort()
                self._quic.stop_stream(event.stream_id, 0x101)
                self._quic.reset_stream(event.stream_id, 0x101)
                self.transmit()
                return

        if event.stream_ended:
            session.close()

    def _send_error(self, stream_id: int, status_code: int) -> None:
        """Sends a simple H3 error response and closes the stream."""
        assert self._http is not None
        self._http.send_headers(
            stream_id=stream_id,
            headers=[
                (b":status", str(status_code).encode()),
                (b"server", SERVER_NAME.encode()),
            ],
            end_stream=True,
        )
        self.transmit()

    def _send_websocket_response(
        self,
        stream_id: int,
        request: _WebSocketH3Request,
        end_stream: bool,
    ) -> None:
        """Sends the H3 response headers prepared by the WebSocket handshaker."""
        assert self._http is not None
        status = request.status if request.status else 403
        response_headers: List[Tuple[bytes, bytes]] = [
            (b":status", str(status).encode()),
            (b"server", SERVER_NAME.encode()),
        ]
        headers_out = list(request.headers_out.items())
        headers_out.extend(request.extra_headers)
        for name, value in headers_out:
            header_name = (name.decode("ascii")
                           if isinstance(name, bytes) else str(name)).lower()
            if header_name in _H3_FORBIDDEN_RESPONSE_HEADERS:
                continue
            header_value = (value if isinstance(value, bytes)
                            else str(value).encode("latin-1"))
            response_headers.append((header_name.encode(), header_value))

        self._http.send_headers(
            stream_id=stream_id,
            headers=response_headers,
            end_stream=end_stream)
        self.transmit()

    def _close_all_sessions(self) -> None:
        """Closes all WebSocket sessions active on this QUIC connection."""
        for task in self._queued_workers.values():
            task.cancel()
        self._queued_workers.clear()
        for session in list(self._sessions.values()):
            session.abort()
        self._sessions.clear()


class SessionTicketStore:
    """Simple in-memory store for TLS session tickets."""

    def __init__(self) -> None:
        self.tickets: Dict[bytes, SessionTicket] = {}

    def add(self, ticket: SessionTicket) -> None:
        self.tickets[ticket.ticket] = ticket

    def pop(self, label: bytes) -> Optional[SessionTicket]:
        return self.tickets.pop(label, None)


class WebSocketH3Server:
    """A WebSocket over HTTP/3 server for testing.

    The server handles WebSocket Extended CONNECT streams and also serves the
    test page/resources from the same H3 origin. The route-related parameters
    are passed to wptserve so regular WPT requests use the same routing,
    rewrite, and template-substitution behavior as the HTTP/HTTPS/H2 servers.

    :param host: Host to bind.
    :param port: UDP port to bind for QUIC.
    :param ws_doc_root: Directory containing WebSocket *_wsh.py handlers.
    :param cert_path: Path to the TLS certificate file.
    :param key_path: Path to the TLS private key file.
    :param doc_root: Directory containing WPT test files and resources.
    :param routes: wptserve route table for regular WPT resources.
    :param rewrites: wptserve rewrite rules applied before routing.
    :param config: wptserve config used by requests and template substitution.
    :param logger: Logger object for this server.
    :param max_pending_handshakes: Maximum concurrent handshakes per QUIC
        connection; additional CONNECT streams wait for a slot.
    :param max_active_requests: Maximum concurrent resource handlers per QUIC
        connection; additional requests wait for a slot.
    :param max_unread_websocket_bytes: Maximum bytes waiting to be read by
        the WebSocket handler per stream; exceeding this resets that stream.
    """

    def __init__(
        self,
        host: str,
        port: int,
        ws_doc_root: str,
        cert_path: str,
        key_path: str,
        doc_root: str = "",
        routes: Any = None,
        rewrites: Any = None,
        config: Any = None,
        logger: Optional[logging.Logger] = None,
        max_pending_handshakes: int = DEFAULT_MAX_PENDING_HANDSHAKES,
        max_active_requests: int = DEFAULT_MAX_ACTIVE_REQUESTS,
        max_unread_websocket_bytes: int = DEFAULT_MAX_UNREAD_WEBSOCKET_BYTES,
    ) -> None:
        for name, value in (
            ("max_pending_handshakes", max_pending_handshakes),
            ("max_active_requests", max_active_requests),
            ("max_unread_websocket_bytes", max_unread_websocket_bytes),
        ):
            if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
                raise ValueError(f"{name} must be a positive integer")
        self.host = host
        self.port = port
        self.ws_doc_root = ws_doc_root
        self.cert_path = cert_path
        self.key_path = key_path
        self.doc_root = doc_root
        self.routes = routes
        self.rewrites = rewrites
        self.config = config
        self.max_pending_handshakes = max_pending_handshakes
        self.max_active_requests = max_active_requests
        self.max_unread_websocket_bytes = max_unread_websocket_bytes
        self._logger = logger if logger is not None else _logger
        self._router: Any = None
        self._h3_server_adapter: Optional[_H3ServerAdapter] = None
        self._h3_request_cls: Any = None
        self._h3_response_cls: Any = None
        self.started = False
        self.loop: Optional[asyncio.AbstractEventLoop] = None
        self.server_thread: Optional[threading.Thread] = None
        self._server: Any = None
        self._startup_complete = threading.Event()
        self._startup_error: Optional[BaseException] = None

        if self.doc_root:
            from wptserve.router import Router
            route_list = self.routes
            if route_list is None:
                from wptserve import routes as default_routes
                route_list = default_routes.routes

            rewriter = None
            if self.rewrites is not None:
                from wptserve.server import RequestRewriter
                request_rewriter_class: Any = RequestRewriter
                rewriter = request_rewriter_class(self.rewrites)
            router_class: Any = Router
            self._router = router_class(self.doc_root, route_list)
            self._h3_server_adapter = _H3ServerAdapter(
                self._router, self.host, self.port, rewriter, self.config)

        (self._h3_request_cls,
         self._h3_response_cls) = create_h3_request_response_classes()

    def start(self) -> None:
        """Start the server."""
        self._startup_complete.clear()
        self._startup_error = None
        self.server_thread = threading.Thread(
            target=self._start_on_server_thread, daemon=True)
        self.server_thread.start()
        self._startup_complete.wait()
        startup_error: Optional[BaseException] = self._startup_error
        if startup_error is not None:
            self.server_thread.join()
            raise startup_error
        self.started = True

    def _start_on_server_thread(self) -> None:
        secrets_log_file = None
        try:
            if "SSLKEYLOGFILE" in os.environ:
                try:
                    secrets_log_file = open(os.environ["SSLKEYLOGFILE"], "a")
                except OSError as error:
                    self._logger.warning(
                        "Failed to open SSLKEYLOGFILE: %s", error)

            if secrets_log_file is not None:
                configuration = QuicConfiguration(
                    alpn_protocols=H3_ALPN,
                    is_client=False,
                    max_datagram_frame_size=65536,
                    secrets_log_file=secrets_log_file,
                )
            else:
                configuration = QuicConfiguration(
                    alpn_protocols=H3_ALPN,
                    is_client=False,
                    max_datagram_frame_size=65536,
                )

            configuration.load_cert_chain(
                Path(self.cert_path), Path(self.key_path))

            ticket_store = SessionTicketStore()

            if sys.platform == "win32":
                asyncio.set_event_loop_policy(
                    asyncio.WindowsSelectorEventLoopPolicy())

            self.loop = asyncio.new_event_loop()
            asyncio.set_event_loop(self.loop)

            self._server = self.loop.run_until_complete(
                serve(
                    self.host,
                    self.port,
                    configuration=configuration,
                    create_protocol=self._create_protocol,
                    session_ticket_fetcher=ticket_store.pop,
                    session_ticket_handler=ticket_store.add,
                ))
            self._startup_complete.set()
            self.loop.run_forever()
        except BaseException as error:
            self._startup_error = error
            self._startup_complete.set()
            self._logger.exception("WebSocket/H3 server thread failed")
        finally:
            if self.loop is not None:
                if self._server is not None:
                    for protocol in set(getattr(
                            self._server, "_protocols", {}).values()):
                        protocol._close_all_sessions()
                    self._server.close()
                    self.loop.run_until_complete(asyncio.sleep(0))
                    self._server = None
                pending = asyncio.all_tasks(self.loop)
                for task in pending:
                    task.cancel()
                if pending:
                    self.loop.run_until_complete(
                        asyncio.gather(*pending, return_exceptions=True))
                self.loop.close()
            if secrets_log_file is not None:
                secrets_log_file.close()

    def _create_protocol(self, *args: Any, **kwargs: Any) -> WebSocketH3Protocol:
        return WebSocketH3Protocol(
            *args,
            ws_doc_root=self.ws_doc_root,
            router=self._router,
            h3_server_adapter=self._h3_server_adapter,
            h3_request_cls=self._h3_request_cls,
            h3_response_cls=self._h3_response_cls,
            logger=self._logger,
            max_pending_handshakes=self.max_pending_handshakes,
            max_active_requests=self.max_active_requests,
            max_unread_websocket_bytes=self.max_unread_websocket_bytes,
            **kwargs)

    def stop(self) -> None:
        """Stop the server."""
        if self.started and self.loop is not None:
            self.loop.call_soon_threadsafe(
                lambda: asyncio.create_task(self._stop_on_server_thread()))
            if self.server_thread is not None:
                self.server_thread.join()
            self._logger.info("Stopped %s on %s:%d",
                              SERVER_NAME, self.host, self.port)
        self.started = False

    async def _stop_on_server_thread(self) -> None:
        assert self.loop is not None
        try:
            if self._server is not None:
                # Stop accepting peers before draining work on existing connections.
                self._server._transport.close()
                protocols = set(getattr(self._server, "_protocols", {}).values())
                workers = [task for protocol in protocols
                           for task in protocol._workers]
                if workers:
                    _, pending = await asyncio.wait(workers, timeout=10)
                    if pending:
                        self._logger.warning(
                            "Timed out waiting for %d WebSocket/H3 requests",
                            len(pending))
                self._server.close()
                for protocol in protocols:
                    protocol._close_all_sessions()
        finally:
            self.loop.call_soon(self.loop.stop)


def server_is_running(host: str, port: int, timeout: float) -> bool:
    """Open a temporary QUIC connection to check if the server is reachable."""
    try:
        loop = asyncio.get_event_loop()
    except RuntimeError:
        loop = asyncio.new_event_loop()
    return loop.run_until_complete(
        _server_is_running_with_timeout(host, port, timeout))


async def _server_is_running_with_timeout(
    host: str, port: int, timeout: float
) -> bool:
    """Run the QUIC readiness probe with the caller-provided timeout."""
    try:
        await asyncio.wait_for(_connect_to_server(host, port), timeout=timeout)
    except asyncio.TimeoutError:
        _logger.warning(
            "Failed to connect to WebSocket over HTTP/3 server at %s:%d",
            host, port)
        return False
    return True


def _close_unusable_writer(reader: asyncio.StreamReader,
                           writer: asyncio.StreamWriter) -> None:
    """Close peer-initiated unidirectional streams opened during the probe."""
    stream_id = cast(QuicStreamAdapter, writer.transport).stream_id
    if (stream_is_unidirectional(stream_id) and
            not stream_is_client_initiated(stream_id)):
        with contextlib.suppress(ValueError):
            writer.close()


async def _connect_to_server(host: str, port: int) -> None:
    """Connect to the H3 server and ping it to verify it is accepting QUIC."""
    configuration = QuicConfiguration(
        alpn_protocols=H3_ALPN,
        is_client=True,
        verify_mode=ssl.CERT_NONE,
    )
    async with connect(host, port, configuration=configuration,
                       stream_handler=_close_unusable_writer) as protocol:
        await protocol.ping()
