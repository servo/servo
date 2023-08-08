# mypy: allow-subclassing-any, no-warn-return-any

import asyncio
import logging
import os
import ssl
import sys
import threading
import traceback
from enum import IntEnum
from urllib.parse import urlparse
from typing import Any, Dict, List, Optional, Tuple

# TODO(bashi): Remove import check suppressions once aioquic dependency is resolved.
from aioquic.buffer import Buffer  # type: ignore
from aioquic.asyncio import QuicConnectionProtocol, serve  # type: ignore
from aioquic.asyncio.client import connect  # type: ignore
from aioquic.h3.connection import H3_ALPN, FrameType, H3Connection, ProtocolError, SettingsError  # type: ignore
from aioquic.h3.events import H3Event, HeadersReceived, WebTransportStreamDataReceived, DatagramReceived, DataReceived  # type: ignore
from aioquic.quic.configuration import QuicConfiguration  # type: ignore
from aioquic.quic.connection import logger as quic_connection_logger  # type: ignore
from aioquic.quic.connection import stream_is_unidirectional
from aioquic.quic.events import QuicEvent, ProtocolNegotiated, ConnectionTerminated, StreamReset  # type: ignore
from aioquic.tls import SessionTicket  # type: ignore

from tools import localpaths  # noqa: F401
from wptserve import stash
from .capsule import H3Capsule, H3CapsuleDecoder, CapsuleType

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

# Set aioquic's log level to WARNING to suppress some INFO logs which are
# recorded every connection close.
quic_connection_logger.setLevel(logging.WARNING)


class H3DatagramSetting(IntEnum):
    # https://datatracker.ietf.org/doc/html/draft-ietf-masque-h3-datagram-04#section-8.1
    DRAFT04 = 0xffd277
    # https://datatracker.ietf.org/doc/html/rfc9220#section-5-2.2.1
    RFC = 0x33


class WebTransportHttp3Setting(IntEnum):
    # https://datatracker.ietf.org/doc/html/draft-ietf-webtrans-http3-07#section-8.2
    WEBTRANSPORT_MAX_SESSIONS_DRAFT07 = 0xc671706a


class H3ConnectionWithDatagram(H3Connection):
    """
    A H3Connection subclass, to make it work with the latest
    HTTP Datagram protocol.
    """
    # https://datatracker.ietf.org/doc/html/rfc9220#name-iana-considerations
    ENABLE_CONNECT_PROTOCOL = 0x08

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        super().__init__(*args, **kwargs)
        self._datagram_setting: Optional[H3DatagramSetting] = None

    def _validate_settings(self, settings: Dict[int, int]) -> None:
        # aioquic doesn't recognize the RFC version of HTTP Datagrams yet.
        # Intentionally don't call `super()._validate_settings(settings)` since
        # it raises a SettingsError when only the RFC version is negotiated.
        if settings.get(H3DatagramSetting.RFC) == 1:
            self._datagram_setting = H3DatagramSetting.RFC
        elif settings.get(H3DatagramSetting.DRAFT04) == 1:
            self._datagram_setting = H3DatagramSetting.DRAFT04

        if self._datagram_setting is None:
            raise SettingsError("HTTP Datagrams support required")

    def _get_local_settings(self) -> Dict[int, int]:
        settings = super()._get_local_settings()
        settings[H3DatagramSetting.RFC] = 1
        settings[H3DatagramSetting.DRAFT04] = 1
        settings[H3ConnectionWithDatagram.ENABLE_CONNECT_PROTOCOL] = 1
        # This connection can handle only one WebTransport session.
        settings[WebTransportHttp3Setting.WEBTRANSPORT_MAX_SESSIONS_DRAFT07] = 1
        return settings

    @property
    def datagram_setting(self) -> Optional[H3DatagramSetting]:
        return self._datagram_setting


class WebTransportH3Protocol(QuicConnectionProtocol):
    def __init__(self, *args: Any, **kwargs: Any) -> None:
        super().__init__(*args, **kwargs)
        self._handler: Optional[Any] = None
        self._http: Optional[H3ConnectionWithDatagram] = None
        self._session_stream_id: Optional[int] = None
        self._close_info: Optional[Tuple[int, bytes]] = None
        self._capsule_decoder_for_session_stream: H3CapsuleDecoder =\
            H3CapsuleDecoder()
        self._allow_calling_session_closed = True
        self._allow_datagrams = False

    def quic_event_received(self, event: QuicEvent) -> None:
        if isinstance(event, ProtocolNegotiated):
            self._http = H3ConnectionWithDatagram(
                self._quic, enable_webtransport=True)
            if self._http.datagram_setting != H3DatagramSetting.DRAFT04:
                self._allow_datagrams = True

        if self._http is not None:
            for http_event in self._http.handle_event(event):
                self._h3_event_received(http_event)

        if isinstance(event, ConnectionTerminated):
            self._call_session_closed(close_info=None, abruptly=True)
        if isinstance(event, StreamReset):
            if self._handler:
                self._handler.stream_reset(event.stream_id, event.error_code)

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
            origin = headers.get(b"origin")
            # Accept any Origin but the client must send it.
            if method == b"CONNECT" and protocol == b"webtransport" and origin:
                self._session_stream_id = event.stream_id
                self._handshake_webtransport(event, headers)
            else:
                status_code = 404 if origin else 403
                self._send_error_response(event.stream_id, status_code)

        if isinstance(event, DataReceived) and\
           self._session_stream_id == event.stream_id:
            if self._http and not self._http.datagram_setting and\
               len(event.data) > 0:
                raise ProtocolError('Unexpected data on the session stream')
            self._receive_data_on_session_stream(
                event.data, event.stream_ended)
        elif self._handler is not None:
            if isinstance(event, WebTransportStreamDataReceived):
                self._handler.stream_data_received(
                    stream_id=event.stream_id,
                    data=event.data,
                    stream_ended=event.stream_ended)
            elif isinstance(event, DatagramReceived):
                if self._allow_datagrams:
                    self._handler.datagram_received(data=event.data)

    def _receive_data_on_session_stream(self, data: bytes, fin: bool) -> None:
        assert self._http is not None
        if len(data) > 0:
            self._capsule_decoder_for_session_stream.append(data)
        if fin:
            self._capsule_decoder_for_session_stream.final()
        for capsule in self._capsule_decoder_for_session_stream:
            if self._close_info is not None:
                raise ProtocolError((
                    "Receiving a capsule with type = {} after receiving " +
                    "CLOSE_WEBTRANSPORT_SESSION").format(capsule.type))
            assert self._http.datagram_setting is not None
            if self._http.datagram_setting == H3DatagramSetting.RFC:
                self._receive_h3_datagram_rfc_capsule_data(
                    capsule=capsule, fin=fin)
            elif self._http.datagram_setting == H3DatagramSetting.DRAFT04:
                self._receive_h3_datagram_draft04_capsule_data(
                    capsule=capsule, fin=fin)

    def _receive_h3_datagram_rfc_capsule_data(self, capsule: H3Capsule, fin: bool) -> None:
        if capsule.type == CapsuleType.DATAGRAM_RFC:
            raise ProtocolError(
                f"Unimplemented capsule type: {capsule.type}")
        elif capsule.type == CapsuleType.CLOSE_WEBTRANSPORT_SESSION:
            self._set_close_info_and_may_close_session(
                data=capsule.data, fin=fin)
        else:
            # Ignore unknown capsules.
            return

    def _receive_h3_datagram_draft04_capsule_data(
            self, capsule: H3Capsule, fin: bool) -> None:
        if capsule.type in {CapsuleType.DATAGRAM_DRAFT04,
                            CapsuleType.REGISTER_DATAGRAM_CONTEXT_DRAFT04,
                            CapsuleType.CLOSE_DATAGRAM_CONTEXT_DRAFT04}:
            raise ProtocolError(
                f"Unimplemented capsule type: {capsule.type}")
        if capsule.type in {CapsuleType.REGISTER_DATAGRAM_NO_CONTEXT_DRAFT04,
                            CapsuleType.CLOSE_WEBTRANSPORT_SESSION}:
            # We'll handle this case below.
            pass
        else:
            # We should ignore unknown capsules.
            return

        if capsule.type == CapsuleType.REGISTER_DATAGRAM_NO_CONTEXT_DRAFT04:
            buffer = Buffer(data=capsule.data)
            format_type = buffer.pull_uint_var()
            # https://ietf-wg-webtrans.github.io/draft-ietf-webtrans-http3/draft-ietf-webtrans-http3.html#name-datagram-format-type
            WEBTRANPORT_FORMAT_TYPE = 0xff7c00
            if format_type != WEBTRANPORT_FORMAT_TYPE:
                raise ProtocolError(
                    "Unexpected datagram format type: {}".format(
                        format_type))
            self._allow_datagrams = True
        elif capsule.type == CapsuleType.CLOSE_WEBTRANSPORT_SESSION:
            self._set_close_info_and_may_close_session(
                data=capsule.data, fin=fin)

    def _set_close_info_and_may_close_session(
            self, data: bytes, fin: bool) -> None:
        buffer = Buffer(data=data)
        code = buffer.pull_uint32()
        # 4 bytes for the uint32.
        reason = buffer.pull_bytes(len(data) - 4)
        # TODO(bashi): Make sure `reason` is a UTF-8 text.
        self._close_info = (code, reason)
        if fin:
            self._call_session_closed(self._close_info, abruptly=False)

    def _send_error_response(self, stream_id: int, status_code: int) -> None:
        assert self._http is not None
        headers = [(b":status", str(status_code).encode()),
                   (b"server", SERVER_NAME.encode())]
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
        except OSError:
            self._send_error_response(event.stream_id, 404)
            return

        response_headers = [
            (b"server", SERVER_NAME.encode()),
            (b"sec-webtransport-http3-draft", b"draft02"),
        ]
        self._handler.connect_received(response_headers=response_headers)

        status_code = None
        for name, value in response_headers:
            if name == b":status":
                status_code = value
                response_headers.remove((b":status", status_code))
                response_headers.insert(0, (b":status", status_code))
                break
        if not status_code:
            response_headers.insert(0, (b":status", b"200"))
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

    def _call_session_closed(
            self, close_info: Optional[Tuple[int, bytes]],
            abruptly: bool) -> None:
        allow_calling_session_closed = self._allow_calling_session_closed
        self._allow_calling_session_closed = False
        if self._handler and allow_calling_session_closed:
            self._handler.session_closed(close_info, abruptly)


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
        self._dict_for_handlers: Dict[str, Any] = {}

    @property
    def stash(self) -> stash.Stash:
        """A Stash object for storing cross-session state."""
        if self._stash is None:
            address, authkey = stash.load_env_config()  # type: ignore
            self._stash = stash.Stash(self._stash_path, address, authkey)  # type: ignore
        return self._stash

    @property
    def dict_for_handlers(self) -> Dict[str, Any]:
        """A dictionary that handlers can attach arbitrary data."""
        return self._dict_for_handlers

    def stream_is_unidirectional(self, stream_id: int) -> bool:
        """Return True if the stream is unidirectional."""
        return stream_is_unidirectional(stream_id)

    def close(self, close_info: Optional[Tuple[int, bytes]]) -> None:
        """
        Close the session.

        :param close_info The close information to send.
        """
        self._protocol._allow_calling_session_closed = False
        assert self._protocol._session_stream_id is not None
        session_stream_id = self._protocol._session_stream_id
        if close_info is not None:
            code = close_info[0]
            reason = close_info[1]
            buffer = Buffer(capacity=len(reason) + 4)
            buffer.push_uint32(code)
            buffer.push_bytes(reason)
            capsule =\
                H3Capsule(CapsuleType.CLOSE_WEBTRANSPORT_SESSION, buffer.data)
            self._http.send_data(
                session_stream_id, capsule.encode(), end_stream=False)

        self._http.send_data(session_stream_id, b'', end_stream=True)
        # TODO(yutakahirano): Reset all other streams.
        # TODO(yutakahirano): Reject future stream open requests
        # We need to wait for the stream data to arrive at the client, and then
        # we need to close the connection. At this moment we're relying on the
        # client's behavior.
        # TODO(yutakahirano): Implement the above.

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
        # TODO(bashi): Remove this workaround when aioquic supports receiving
        # data on server-initiated bidirectional streams.
        stream = self._http._get_or_create_stream(stream_id)
        assert stream.frame_type is None
        assert stream.session_id is None
        stream.frame_type = FrameType.WEBTRANSPORT_STREAM
        stream.session_id = self.session_id
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
        if not self._protocol._allow_datagrams:
            _logger.warn(
                "Sending a datagram while that's now allowed - discarding it")
            return
        flow_id = self.session_id
        if self._http.datagram_setting is not None:
            # We must have a WebTransport Session ID at this point because
            # an extended CONNECT request is already received.
            assert self._protocol._session_stream_id is not None
            # TODO(yutakahirano): Make sure if this is the correct logic.
            # Chrome always use 0 for the initial stream and the initial flow
            # ID, we cannot check the correctness with it.
            flow_id = self._protocol._session_stream_id // 4
        self._http.send_datagram(flow_id=flow_id, data=data)

    def stop_stream(self, stream_id: int, code: int) -> None:
        """
        Send a STOP_SENDING frame to the given stream.
        :param code: the reason of the error.
        """
        self._http._quic.stop_stream(stream_id, code)

    def reset_stream(self, stream_id: int, code: int) -> None:
        """
        Send a RESET_STREAM frame to the given stream.
        :param code: the reason of the error.
        """
        self._http._quic.reset_stream(stream_id, code)


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

    def session_closed(
            self,
            close_info: Optional[Tuple[int, bytes]],
            abruptly: bool) -> None:
        self._run_callback(
            "session_closed", self._session, close_info, abruptly=abruptly)

    def stream_reset(self, stream_id: int, error_code: int) -> None:
        self._run_callback(
            "stream_reset", self._session, stream_id, error_code)


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
        secrets_log_file = None
        if "SSLKEYLOGFILE" in os.environ:
            try:
                secrets_log_file = open(os.environ["SSLKEYLOGFILE"], "a")
            except Exception as e:
                _logger.warn(str(e))

        configuration = QuicConfiguration(
            alpn_protocols=H3_ALPN,
            is_client=False,
            max_datagram_frame_size=65536,
            secrets_log_file=secrets_log_file,
        )

        _logger.info("Starting WebTransport over HTTP/3 server on %s:%s",
                     self.host, self.port)

        configuration.load_cert_chain(self.cert_path, self.key_path)

        ticket_store = SessionTicketStore()

        # On Windows, the default event loop is ProactorEventLoop but it
        # doesn't seem to work when aioquic detects a connection loss.
        # Use SelectorEventLoop to work around the problem.
        if sys.platform == "win32":
            asyncio.set_event_loop_policy(
                asyncio.WindowsSelectorEventLoopPolicy())
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


def server_is_running(host: str, port: int, timeout: float) -> bool:
    """
    Check the WebTransport over HTTP/3 server is running at the given `host` and
    `port`.
    """
    loop = asyncio.get_event_loop()
    return loop.run_until_complete(_connect_server_with_timeout(host, port, timeout))


async def _connect_server_with_timeout(host: str, port: int, timeout: float) -> bool:
    try:
        await asyncio.wait_for(_connect_to_server(host, port), timeout=timeout)
    except asyncio.TimeoutError:
        _logger.warning("Failed to connect WebTransport over HTTP/3 server")
        return False
    return True


async def _connect_to_server(host: str, port: int) -> None:
    configuration = QuicConfiguration(
        alpn_protocols=H3_ALPN,
        is_client=True,
        verify_mode=ssl.CERT_NONE,
    )

    async with connect(host, port, configuration=configuration) as protocol:
        await protocol.ping()
