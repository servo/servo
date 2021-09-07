import asyncio
import logging
import os
import threading
import traceback
from urllib.parse import urlparse
from typing import Any, Dict, List, Optional, Tuple

# TODO(bashi): Remove import check suppressions once aioquic dependency is resolved.
from aioquic.asyncio import QuicConnectionProtocol, serve  # type: ignore
from aioquic.h3.connection import H3_ALPN, H3Connection  # type: ignore
from aioquic.h3.events import H3Event, HeadersReceived, WebTransportStreamDataReceived, DatagramReceived  # type: ignore
from aioquic.quic.configuration import QuicConfiguration  # type: ignore
from aioquic.quic.connection import stream_is_unidirectional  # type: ignore
from aioquic.quic.events import QuicEvent, ProtocolNegotiated  # type: ignore
from aioquic.tls import SessionTicket  # type: ignore
from aioquic.quic.packet import QuicErrorCode  # type: ignore

from tools.wptserve.wptserve import stash  # type: ignore

"""
A WebTransport over HTTP/3 server for testing.

The server interprets the underlying protocols (WebTransport, HTTP/3 and QUIC)
and passes events to a particular webtransport handler. From the standpoint of
test authors, a webtransport handler is a Python script which contains some
callback functions. See handler.py for available callbacks.
"""

SERVER_NAME = 'webtransport-h3-server'

_logger: logging.Logger = logging.getLogger(__name__)
_doc_root: str = ""


class WebTransportH3Protocol(QuicConnectionProtocol):
    def __init__(self, *args: Any, **kwargs: Any) -> None:
        super().__init__(*args, **kwargs)
        self._handler: Optional[Any] = None
        self._http: Optional[H3Connection] = None

    def quic_event_received(self, event: QuicEvent) -> None:
        if isinstance(event, ProtocolNegotiated):
            self._http = H3Connection(self._quic, enable_webtransport=True)

        if self._http is not None:
            for http_event in self._http.handle_event(event):
                self._h3_event_received(http_event)

    def _h3_event_received(self, event: H3Event) -> None:
        if isinstance(event, HeadersReceived):
            # Convert from List[Tuple[bytes, bytes]] to Dict[bytes, bytes].
            # Only the last header will be kept when there are duplicate
            # headers.
            headers = {}
            for header, value in event.headers:
                headers[header] = value

            method = headers.get(b":method")
            protocol = headers.get(b":protocol")
            if method == b"CONNECT" and protocol == b"webtransport":
                self._handshake_webtransport(event, headers)
            else:
                self._send_error_response(event.stream_id, 400)

        if self._handler is not None:
            if isinstance(event, WebTransportStreamDataReceived):
                self._handler.stream_data_received(stream_id=event.stream_id,
                                                   data=event.data,
                                                   stream_ended=event.stream_ended)
            elif isinstance(event, DatagramReceived):
                self._handler.datagram_received(data=event.data)

    def _send_error_response(self, stream_id: int, status_code: int) -> None:
        assert self._http is not None
        headers = [(b"server", SERVER_NAME.encode()),
                   (b":status", str(status_code).encode())]
        self._http.send_headers(stream_id=stream_id,
                                headers=headers,
                                end_stream=True)

    def _handshake_webtransport(self, event: HeadersReceived,
                                request_headers: Dict[bytes, bytes]) -> None:
        assert self._http is not None
        path = request_headers.get(b":path")
        if path is None:
            # `:path` must be provided.
            self._send_error_response(event.stream_id, 400)
            return

        # Create a handler using `:path`.
        try:
            self._handler = self._create_event_handler(
                session_id=event.stream_id,
                path=path,
                request_headers=event.headers)
        except IOError:
            self._send_error_response(event.stream_id, 404)
            return

        response_headers = [
            (b"server", SERVER_NAME.encode()),
        ]
        self._handler.connect_received(response_headers=response_headers)

        status_code = None
        for name, value in response_headers:
            if name == b":status":
                status_code = value
                break
        if not status_code:
            response_headers.append((b":status", b"200"))
        self._http.send_headers(stream_id=event.stream_id,
                                headers=response_headers)

        if status_code is None or status_code == b"200":
            self._handler.session_established()

    def _create_event_handler(self, session_id: int, path: bytes,
                              request_headers: List[Tuple[bytes, bytes]]) -> Any:
        parsed = urlparse(path.decode())
        file_path = os.path.join(_doc_root, parsed.path.lstrip("/"))
        callbacks = {"__file__": file_path}
        with open(file_path) as f:
            exec(compile(f.read(), path, "exec"), callbacks)
        session = WebTransportSession(self, session_id, request_headers)
        return WebTransportEventHandler(session, callbacks)


class WebTransportSession:
    """
    A WebTransport session.
    """

    def __init__(self, protocol: WebTransportH3Protocol, session_id: int,
                 request_headers: List[Tuple[bytes, bytes]]) -> None:
        self.session_id = session_id
        self.request_headers = request_headers

        self._protocol: WebTransportH3Protocol = protocol
        self._http: H3Connection = protocol._http

        # Use the a shared default path for all handlers so that different
        # WebTransport sessions can access the same store easily.
        self._stash_path = '/webtransport/handlers'
        self._stash: Optional[stash.Stash] = None

    @property
    def stash(self) -> stash.Stash:
        """A Stash object for storing cross-session state."""
        if self._stash is None:
            address, authkey = stash.load_env_config()
            self._stash = stash.Stash(self._stash_path, address, authkey)
        return self._stash

    def stream_is_unidirectional(self, stream_id: int) -> bool:
        """Return True if the stream is unidirectional."""
        return stream_is_unidirectional(stream_id)

    def close(self,
              error_code: int = QuicErrorCode.NO_ERROR,
              reason_phrase: str = "") -> None:
        """
        Close the session.

        :param error_code: An error code indicating why the session is
                           being closed.
        :param reason_phrase: A human readable explanation of why the
                              session is being closed.
        """
        self._http._quic.close(error_code=error_code,
                               reason_phrase=reason_phrase)
        self._protocol.transmit()

    def create_unidirectional_stream(self) -> int:
        """
        Create a unidirectional WebTransport stream and return the stream ID.
        """
        return self._http.create_webtransport_stream(
            session_id=self.session_id, is_unidirectional=True)

    def create_bidirectional_stream(self) -> int:
        """
        Create a bidirectional WebTransport stream and return the stream ID.
        """
        stream_id = self._http.create_webtransport_stream(
            session_id=self.session_id, is_unidirectional=False)
        return stream_id

    def send_stream_data(self,
                         stream_id: int,
                         data: bytes,
                         end_stream: bool = False) -> None:
        """
        Send data on the specific stream.

        :param stream_id: The stream ID on which to send the data.
        :param data: The data to send.
        :param end_stream: If set to True, the stream will be closed.
        """
        self._http._quic.send_stream_data(stream_id=stream_id,
                                          data=data,
                                          end_stream=end_stream)

    def send_datagram(self, data: bytes) -> None:
        """
        Send data using a datagram frame.

        :param data: The data to send.
        """
        self._http.send_datagram(flow_id=self.session_id, data=data)


class WebTransportEventHandler:
    def __init__(self, session: WebTransportSession,
                 callbacks: Dict[str, Any]) -> None:
        self._session = session
        self._callbacks = callbacks

    def _run_callback(self, callback_name: str,
                      *args: Any, **kwargs: Any) -> None:
        if callback_name not in self._callbacks:
            return
        try:
            self._callbacks[callback_name](*args, **kwargs)
        except Exception as e:
            _logger.warn(str(e))
            traceback.print_exc()

    def connect_received(self, response_headers: List[Tuple[bytes,
                                                            bytes]]) -> None:
        self._run_callback("connect_received", self._session.request_headers,
                           response_headers)

    def session_established(self) -> None:
        self._run_callback("session_established", self._session)

    def stream_data_received(self, stream_id: int, data: bytes,
                             stream_ended: bool) -> None:
        self._run_callback("stream_data_received", self._session, stream_id,
                           data, stream_ended)

    def datagram_received(self, data: bytes) -> None:
        self._run_callback("datagram_received", self._session, data)


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


class WebTransportH3Server:
    """
    A WebTransport over HTTP/3 for testing.

    :param host: Host from which to serve.
    :param port: Port from which to serve.
    :param doc_root: Document root for serving handlers.
    :param cert_path: Path to certificate file to use.
    :param key_path: Path to key file to use.
    :param logger: a Logger object for this server.
    """
    def __init__(self, host: str, port: int, doc_root: str, cert_path: str,
                 key_path: str, logger: Optional[logging.Logger]) -> None:
        self.host = host
        self.port = port
        self.doc_root = doc_root
        self.cert_path = cert_path
        self.key_path = key_path
        self.started = False
        global _doc_root
        _doc_root = self.doc_root
        global _logger
        if logger is not None:
            _logger = logger

    def start(self) -> None:
        """Start the server."""
        self.server_thread = threading.Thread(
            target=self._start_on_server_thread, daemon=True)
        self.server_thread.start()
        self.started = True

    def _start_on_server_thread(self) -> None:
        configuration = QuicConfiguration(
            alpn_protocols=H3_ALPN,
            is_client=False,
            max_datagram_frame_size=65536,
        )

        _logger.info("Starting WebTransport over HTTP/3 server on %s:%s",
                     self.host, self.port)

        configuration.load_cert_chain(self.cert_path, self.key_path)

        ticket_store = SessionTicketStore()

        self.loop = asyncio.new_event_loop()
        asyncio.set_event_loop(self.loop)
        self.loop.run_until_complete(
            serve(
                self.host,
                self.port,
                configuration=configuration,
                create_protocol=WebTransportH3Protocol,
                session_ticket_fetcher=ticket_store.pop,
                session_ticket_handler=ticket_store.add,
            ))
        self.loop.run_forever()

    def stop(self) -> None:
        """Stop the server."""
        if self.started:
            asyncio.run_coroutine_threadsafe(self._stop_on_server_thread(),
                                             self.loop)
            self.server_thread.join()
            _logger.info("Stopped WebTransport over HTTP/3 server on %s:%s",
                         self.host, self.port)
        self.started = False

    async def _stop_on_server_thread(self) -> None:
        self.loop.stop()
