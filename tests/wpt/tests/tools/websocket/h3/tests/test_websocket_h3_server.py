# mypy: allow-untyped-defs

import asyncio
import importlib.util
import socket
import threading
import time
import unittest
from io import BytesIO
from types import SimpleNamespace
from unittest import mock

import pytest
from pywebsocket3.handshake.base import AbortedByUserException


if importlib.util.find_spec('aioquic'):
    has_aioquic = True
    from aioquic.h3.connection import H3Connection
    from aioquic.quic.events import ConnectionTerminated
    from aioquic.quic.events import StreamReset

    from .. import h3_wptserve_adapter
    from .. import websocket_h3_server
    from .. import websocket_h3_session
else:
    has_aioquic = False


class _FakeH3:
    def __init__(self):
        self.headers = []
        self.data = []

    def send_headers(self, stream_id, headers, end_stream):
        self.headers.append((stream_id, headers, end_stream))

    def send_data(self, stream_id, data, end_stream):
        self.data.append((stream_id, data, end_stream))


class _FakeProtocol:
    def __init__(self):
        self._http = _FakeH3()
        self._sessions = {}
        self.transmits = 0

    def transmit(self):
        self.transmits += 1


def _make_headers_event(stream_id, method, protocol=None, stream_ended=False):
    headers = [(b':method', method)]
    if protocol is not None:
        headers.append((b':protocol', protocol))
    return SimpleNamespace(stream_id=stream_id, headers=headers,
                           stream_ended=stream_ended)


def _make_websocket_connect_event(stream_id=7):
    return SimpleNamespace(
        stream_id=stream_id,
        stream_ended=False,
        headers=[
            (b':method', b'CONNECT'),
            (b':protocol', b'websocket'),
            (b':authority', b'web-platform.test:11001'),
            (b':path', b'/echo'),
            (b'sec-websocket-version', b'13'),
        ])


def _make_websocket_h3_protocol():
    return object.__new__(websocket_h3_server.WebSocketH3Protocol)


def _make_protocol_for_websocket_connect():
    protocol = _make_websocket_h3_protocol()
    protocol._http = _FakeH3()
    protocol._sessions = {}
    protocol._workers = set()
    protocol._queued_workers = {}
    protocol._pending_handshakes = 0
    protocol._handshake_slots = asyncio.Semaphore(
        websocket_h3_server.DEFAULT_MAX_PENDING_HANDSHAKES)
    protocol._request_slots = asyncio.Semaphore(
        websocket_h3_server.DEFAULT_MAX_ACTIVE_REQUESTS)
    protocol._max_unread_websocket_bytes = (
        websocket_h3_session.DEFAULT_MAX_UNREAD_WEBSOCKET_BYTES)
    protocol._ws_doc_root = ''
    protocol._logger = mock.Mock()
    protocol.transmit = mock.Mock()
    protocol._loop = None
    return protocol


def _make_mock_websocket_session():
    return SimpleNamespace(feed_data=mock.Mock(), close=mock.Mock(),
                           abort=mock.Mock())


class WebSocketH3ServerTest(unittest.TestCase):
    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_local_settings_enable_extended_connect(self):
        """The H3 connection advertises support for WebSocket CONNECT streams."""
        connection = object.__new__(
            websocket_h3_server.H3ConnectionForWebSocket)

        with mock.patch.object(H3Connection, '_get_local_settings',
                               return_value={}):
            settings = (
                websocket_h3_server.H3ConnectionForWebSocket
                ._get_local_settings(connection))

        self.assertEqual(
            settings[
                websocket_h3_server
                .H3ConnectionForWebSocket.ENABLE_CONNECT_PROTOCOL],
            1)

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_response_writer_sends_h3_headers_and_data(self):
        """The wptserve response writer sends response headers and body as H3."""
        protocol = _FakeProtocol()
        handler = SimpleNamespace(h3_protocol=protocol, h3_stream_id=3)
        request = SimpleNamespace(method='GET')
        response = SimpleNamespace(request=request, encoding='utf8')
        writer = h3_wptserve_adapter.H3ResponseWriter(handler, response)

        writer.write_headers([
            ('Content-Type', 'text/plain'),
            ('connection', 'keep-alive'),
        ], 200)
        writer.write_data(BytesIO(b'payload'), last=True)

        self.assertEqual(protocol._http.headers, [(
            3,
            [(b':status', b'200'), (b'content-type', b'text/plain')],
            False,
        )])
        self.assertEqual(protocol._http.data, [(3, b'payload', True)])

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_response_writer_preserves_byte_headers(self):
        protocol = _FakeProtocol()
        handler = SimpleNamespace(h3_protocol=protocol, h3_stream_id=3)
        response = SimpleNamespace(request=SimpleNamespace(method='GET'),
                                   encoding='utf8')
        writer = h3_wptserve_adapter.H3ResponseWriter(handler, response)
        writer.write_headers([(b'x-bytes', b'\xff'), ('x-text', '\u00e9')], 200)
        self.assertEqual(protocol._http.headers[0][1][1:],
                         [(b'x-bytes', b'\xff'),
                          (b'x-text', b'\xe9')])

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_websocket_connect_sends_handshake_response(self):
        """A WebSocket CONNECT request sends the handshake response and starts."""
        dispatcher = SimpleNamespace(get_handler_suite=lambda path: object())

        class FakeHandshaker:
            def __init__(self, request, dispatcher):
                self.request = request

            def do_handshake(self):
                self.request.status = 200
                self.request.headers_out['sec-websocket-protocol'] = 'chat'
                self.request.extra_headers.append(('x-extra', 'value'))
                self.request.connection.write(b'early-data')

        async def run():
            protocol = _make_protocol_for_websocket_connect()
            websocket_h3_server.WebSocketH3Protocol._handle_websocket_connect(
                protocol, _make_websocket_connect_event())
            await asyncio.gather(*(task for task in asyncio.all_tasks()
                                   if task is not asyncio.current_task()))
            return protocol

        with mock.patch.object(websocket_h3_server.dispatch, 'Dispatcher',
                               return_value=dispatcher), \
             mock.patch.object(websocket_h3_server, 'WsH3Handshaker',
                               FakeHandshaker), \
             mock.patch.object(
                 websocket_h3_session._WebSocketH3Session, 'start') as start:
            protocol = asyncio.run(run())

        self.assertEqual(protocol._http.headers, [(
            7,
            [
                (b':status', b'200'),
                (b'server', b'websocket-h3-server'),
                (b'sec-websocket-protocol', b'chat'),
                (b'x-extra', b'value'),
            ],
            False,
        )])
        self.assertIn(7, protocol._sessions)
        self.assertEqual(protocol._http.data, [(7, b'early-data', False)])
        start.assert_called_once_with()
        protocol._sessions[7].abort()

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_websocket_response_preserves_multiple_set_cookie_headers(self):
        protocol = _FakeProtocol()
        request = SimpleNamespace(
            status=200,
            headers_out={'sec-websocket-protocol': 'chat'},
            extra_headers=[
                ('Set-Cookie', 'first=one; Path=/'),
                (b'Set-Cookie', b'second=two; Path=/'),
                ('x-extra', 'value'),
            ],
        )

        websocket_h3_server.WebSocketH3Protocol._send_websocket_response(
            protocol, 7, request, end_stream=False)

        self.assertEqual(protocol._http.headers, [(
            7,
            [
                (b':status', b'200'),
                (b'server', b'websocket-h3-server'),
                (b'sec-websocket-protocol', b'chat'),
                (b'set-cookie', b'first=one; Path=/'),
                (b'set-cookie', b'second=two; Path=/'),
                (b'x-extra', b'value'),
            ],
            False,
        )])

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_aborted_handshake_sends_authentication_response(self):
        class AbortingHandshaker:
            def __init__(self, request, dispatcher):
                self.request = request

            def do_handshake(self):
                self.request.status = 401
                self.request.headers_out['www-authenticate'] = (
                    'Basic realm="camelot"')
                raise AbortedByUserException()

        async def run():
            protocol = _make_protocol_for_websocket_connect()
            protocol._handle_websocket_connect(_make_websocket_connect_event())
            await asyncio.gather(*protocol._workers)
            return protocol

        with mock.patch.object(websocket_h3_server.dispatch, 'Dispatcher'), \
             mock.patch.object(websocket_h3_server, 'WsH3Handshaker',
                               AbortingHandshaker):
            protocol = asyncio.run(run())

        self.assertEqual(protocol._http.headers, [(
            7,
            [
                (b':status', b'401'),
                (b'server', b'websocket-h3-server'),
                (b'www-authenticate', b'Basic realm="camelot"'),
            ],
            True,
        )])
        self.assertNotIn(7, protocol._sessions)

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_slow_http_handler_and_response_writer_do_not_block_loop(self):
        class Request:
            method = 'GET'

            def __init__(self, handler):
                pass

            def __enter__(self):
                return self

            def __exit__(self, *args):
                pass

        class Response:
            encoding = 'utf8'

            def __init__(self, handler, request):
                self.request = request
                self.writer = h3_wptserve_adapter.H3ResponseWriter(
                    handler, self)

            def write(self):
                self.writer.write_headers([('x-test', 'ok')], 200)
                self.writer.write_data(b'body', last=True)

        async def run():
            loop = asyncio.get_running_loop()
            loop_thread = threading.get_ident()
            started = threading.Event()
            release = threading.Event()
            protocol = _make_protocol_for_websocket_connect()
            protocol._loop = loop
            protocol._call_on_loop = (
                websocket_h3_server.WebSocketH3Protocol._call_on_loop
                .__get__(protocol))
            protocol._active_requests = 0
            protocol._h3_server_adapter = SimpleNamespace(
                rewriter=SimpleNamespace(rewrite=lambda handler: None))
            protocol._h3_request_cls = Request
            protocol._h3_response_cls = Response

            def handler(request, response):
                self.assertNotEqual(threading.get_ident(), loop_thread)
                started.set()
                self.assertTrue(release.wait(3))

            protocol._router = SimpleNamespace(
                get_handler=lambda request: handler)
            protocol._http.send_headers = mock.Mock(
                wraps=protocol._http.send_headers)
            protocol._http.send_data = mock.Mock(
                wraps=protocol._http.send_data)
            websocket_h3_server.WebSocketH3Protocol._handle_request(
                protocol, _make_headers_event(3, b'GET'))
            try:
                self.assertTrue(await asyncio.to_thread(started.wait, 2))
                self.assertEqual(protocol._active_requests, 1)
            finally:
                release.set()
            for _ in range(100):
                if protocol._active_requests == 0:
                    break
                await asyncio.sleep(.01)
            self.assertEqual(protocol._active_requests, 0)
            self.assertEqual(protocol._http.headers[0][1],
                             [(b':status', b'200'), (b'x-test', b'ok')])
            self.assertEqual(protocol._http.data, [(3, b'body', True)])

        asyncio.run(run())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_websocket_connect_without_handler_returns_404(self):
        """A WebSocket CONNECT request without a handler returns 404."""
        dispatcher = SimpleNamespace(get_handler_suite=lambda path: None)

        async def run():
            protocol = _make_protocol_for_websocket_connect()
            websocket_h3_server.WebSocketH3Protocol._handle_websocket_connect(
                protocol, _make_websocket_connect_event())
            await asyncio.gather(*(task for task in asyncio.all_tasks()
                                   if task is not asyncio.current_task()))
            return protocol

        with mock.patch.object(websocket_h3_server.dispatch, 'Dispatcher',
                               return_value=dispatcher):
            protocol = asyncio.run(run())

        self.assertEqual(protocol._http.headers, [(
            7,
            [
                (b':status', b'404'),
                (b'server', b'websocket-h3-server'),
            ],
            True,
        )])
        protocol.transmit.assert_called_once_with()

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_excess_handshakes_wait_for_a_slot(self):
        async def run():
            protocol = _make_protocol_for_websocket_connect()
            protocol._handshake_slots = asyncio.Semaphore(1)
            started = asyncio.Event()
            release = asyncio.Event()
            handled = []

            async def handshake(session, path, dispatcher):
                handled.append(session.stream_id)
                started.set()
                await release.wait()

            protocol._do_websocket_connect = handshake
            with mock.patch.object(websocket_h3_server.dispatch, 'Dispatcher'):
                protocol._handle_websocket_connect(
                    _make_websocket_connect_event())
                await started.wait()
                protocol._handle_websocket_connect(
                    _make_websocket_connect_event(11))
                await asyncio.sleep(0)
                self.assertEqual(handled, [7])
                self.assertEqual(protocol._pending_handshakes, 1)
                self.assertIn(11, protocol._queued_workers)
                self.assertEqual(protocol._http.headers, [])
                release.set()
                await asyncio.gather(*protocol._workers)

            self.assertEqual(handled, [7, 11])
            self.assertEqual(protocol._pending_handshakes, 0)
            protocol._close_all_sessions()

        asyncio.run(run())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_reset_queued_handshake_does_not_run(self):
        async def run():
            protocol = _make_protocol_for_websocket_connect()
            protocol._handshake_slots = asyncio.Semaphore(1)
            protocol._http.handle_event = mock.Mock(return_value=[])
            started = asyncio.Event()
            release = asyncio.Event()
            handled = []

            async def handshake(session, path, dispatcher):
                handled.append(session.stream_id)
                started.set()
                await release.wait()

            protocol._do_websocket_connect = handshake
            with mock.patch.object(websocket_h3_server.dispatch, 'Dispatcher'):
                protocol._handle_websocket_connect(
                    _make_websocket_connect_event())
                await started.wait()
                active = next(iter(protocol._workers))
                protocol._handle_websocket_connect(
                    _make_websocket_connect_event(11))
                await asyncio.sleep(0)
                queued = protocol._queued_workers[11]
                protocol.quic_event_received(
                    StreamReset(error_code=0, stream_id=11))
                release.set()
                await active
                with self.assertRaises(asyncio.CancelledError):
                    await queued

            self.assertEqual(handled, [7])
            self.assertNotIn(11, protocol._sessions)
            self.assertEqual(protocol._pending_handshakes, 0)
            protocol._close_all_sessions()

        asyncio.run(run())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_many_active_websocket_sessions_do_not_reject_new_connect(self):
        dispatcher = SimpleNamespace(get_handler_suite=lambda path: None)

        async def run():
            protocol = _make_protocol_for_websocket_connect()
            protocol._sessions = {stream_id: object()
                                  for stream_id in range(100, 164)}
            websocket_h3_server.WebSocketH3Protocol._handle_websocket_connect(
                protocol, _make_websocket_connect_event())
            self.assertEqual(len(protocol._sessions), 65)
            await asyncio.sleep(0)
            self.assertEqual(protocol._pending_handshakes, 1)
            self.assertEqual(protocol._http.headers, [])
            await asyncio.gather(*protocol._workers)
            return protocol

        with mock.patch.object(websocket_h3_server.dispatch, 'Dispatcher',
                               return_value=dispatcher):
            protocol = asyncio.run(run())
        self.assertEqual(protocol._http.headers[0][1][0], (b':status', b'404'))

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_excess_resource_requests_wait_for_a_slot(self):
        async def run():
            protocol = _make_protocol_for_websocket_connect()
            protocol._router = object()
            protocol._h3_server_adapter = object()
            protocol._request_slots = asyncio.Semaphore(1)
            protocol._active_requests = 0
            started = threading.Event()
            release = threading.Event()
            handled = []

            def serve(event):
                handled.append(event.stream_id)
                started.set()
                self.assertTrue(release.wait(3))

            protocol._serve_request_sync = serve
            protocol._handle_request(_make_headers_event(3, b'GET'))
            try:
                self.assertTrue(await asyncio.to_thread(started.wait, 2))
                protocol._handle_request(_make_headers_event(7, b'GET'))
                await asyncio.sleep(0)
                self.assertEqual(handled, [3])
                self.assertEqual(protocol._active_requests, 1)
                self.assertIn(7, protocol._queued_workers)
                self.assertEqual(protocol._http.headers, [])
            finally:
                release.set()
            await asyncio.gather(*protocol._workers)
            self.assertEqual(handled, [3, 7])
            self.assertEqual(protocol._active_requests, 0)

        asyncio.run(run())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_reset_queued_resource_request_does_not_run(self):
        async def run():
            protocol = _make_protocol_for_websocket_connect()
            protocol._router = object()
            protocol._h3_server_adapter = object()
            protocol._request_slots = asyncio.Semaphore(1)
            protocol._active_requests = 0
            protocol._http.handle_event = mock.Mock(return_value=[])
            started = threading.Event()
            release = threading.Event()
            handled = []

            def serve(event):
                handled.append(event.stream_id)
                started.set()
                self.assertTrue(release.wait(3))

            protocol._serve_request_sync = serve
            protocol._handle_request(_make_headers_event(3, b'GET'))
            try:
                self.assertTrue(await asyncio.to_thread(started.wait, 2))
                active = next(iter(protocol._workers))
                protocol._handle_request(_make_headers_event(7, b'GET'))
                await asyncio.sleep(0)
                queued = protocol._queued_workers[7]
                protocol.quic_event_received(
                    StreamReset(error_code=0, stream_id=7))
            finally:
                release.set()
            await active
            with self.assertRaises(asyncio.CancelledError):
                await queued
            self.assertEqual(handled, [3])
            self.assertEqual(protocol._active_requests, 0)
            self.assertEqual(protocol._queued_workers, {})

        asyncio.run(run())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_server_passes_resource_limits_to_protocol(self):
        server = websocket_h3_server.WebSocketH3Server(
            '127.0.0.1', 0, '', 'unused', 'unused',
            max_pending_handshakes=2, max_active_requests=3,
            max_unread_websocket_bytes=4)
        with mock.patch.object(websocket_h3_server, 'WebSocketH3Protocol') as cls:
            server._create_protocol()
        self.assertEqual(cls.call_args.kwargs['max_pending_handshakes'], 2)
        self.assertEqual(cls.call_args.kwargs['max_active_requests'], 3)
        self.assertEqual(cls.call_args.kwargs['max_unread_websocket_bytes'], 4)
        with self.assertRaisesRegex(ValueError, 'max_pending_handshakes'):
            websocket_h3_server.WebSocketH3Server(
                '127.0.0.1', 0, '', 'unused', 'unused',
                max_pending_handshakes=0)

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_request_headers_are_routed_by_method_and_protocol(self):
        """Request headers select the WebSocket, HTTP, or error path."""
        protocol = _make_websocket_h3_protocol()
        protocol._handle_websocket_connect = mock.Mock()
        protocol._handle_request = mock.Mock()
        protocol._send_error = mock.Mock()

        websocket_event = _make_headers_event(1, b'CONNECT', b'websocket')
        get_event = _make_headers_event(2, b'GET')
        head_event = _make_headers_event(3, b'HEAD')
        connect_event = _make_headers_event(4, b'CONNECT', b'other')
        post_event = _make_headers_event(5, b'POST')

        for event in (
                websocket_event,
                get_event,
                head_event,
                connect_event,
                post_event):
            websocket_h3_server.WebSocketH3Protocol._handle_headers(
                protocol, event)

        protocol._handle_websocket_connect.assert_called_once_with(
            websocket_event)
        protocol._handle_request.assert_has_calls([
            mock.call(get_event),
            mock.call(head_event),
        ])
        self.assertEqual(protocol._send_error.mock_calls, [
            mock.call(4, 501),
            mock.call(5, 405),
        ])

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_stream_reset_and_connection_termination_close_sessions(self):
        """Stream reset closes one session; connection termination closes all."""
        protocol = _make_websocket_h3_protocol()
        protocol._http = SimpleNamespace(
            handle_event=mock.Mock(return_value=[]))
        reset_session = _make_mock_websocket_session()
        first_remaining_session = _make_mock_websocket_session()
        second_remaining_session = _make_mock_websocket_session()
        protocol._sessions = {
            1: reset_session,
            3: first_remaining_session,
            5: second_remaining_session,
        }
        queued = mock.Mock()
        protocol._queued_workers = {7: queued}

        websocket_h3_server.WebSocketH3Protocol.quic_event_received(
            protocol, StreamReset(error_code=0, stream_id=1))

        reset_session.abort.assert_called_once_with()
        first_remaining_session.close.assert_not_called()
        second_remaining_session.close.assert_not_called()
        self.assertEqual(protocol._sessions, {
            3: first_remaining_session,
            5: second_remaining_session,
        })

        websocket_h3_server.WebSocketH3Protocol.quic_event_received(
            protocol,
            ConnectionTerminated(
                error_code=0, frame_type=None, reason_phrase=''))

        first_remaining_session.abort.assert_called_once_with()
        second_remaining_session.abort.assert_called_once_with()
        self.assertEqual(protocol._sessions, {})
        queued.cancel.assert_called_once_with()
        self.assertEqual(protocol._queued_workers, {})

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_multiple_websocket_streams_are_tracked_independently(self):
        """H3 DATA is delivered only to the session for the matching stream."""
        protocol = _make_websocket_h3_protocol()
        first_session = _make_mock_websocket_session()
        second_session = _make_mock_websocket_session()
        protocol._sessions = {
            1: first_session,
            3: second_session,
        }

        websocket_h3_server.WebSocketH3Protocol._handle_data(
            protocol,
            SimpleNamespace(stream_id=3, data=b'frame', stream_ended=False))

        first_session.feed_data.assert_not_called()
        second_session.feed_data.assert_called_once_with(b'frame')
        first_session.close.assert_not_called()
        second_session.close.assert_not_called()
        self.assertEqual(protocol._sessions, {
            1: first_session,
            3: second_session,
        })

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_websocket_data_is_forwarded_to_session(self):
        """Incoming H3 DATA is forwarded to the matching WebSocket session."""
        protocol = _make_websocket_h3_protocol()
        session = _make_mock_websocket_session()
        protocol._sessions = {7: session}

        websocket_h3_server.WebSocketH3Protocol._handle_data(
            protocol,
            SimpleNamespace(stream_id=7, data=b'frame', stream_ended=True))
        session.feed_data.assert_called_once_with(b'frame')
        session.close.assert_called_once_with()
        self.assertEqual(protocol._sessions, {7: session})

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_connect_headers_ending_stream_close_input(self):
        protocol = _make_websocket_h3_protocol()
        protocol._handle_websocket_connect = mock.Mock()
        protocol._sessions = {7: _make_mock_websocket_session()}
        event = _make_headers_event(7, b'CONNECT', b'websocket', True)
        websocket_h3_server.WebSocketH3Protocol._handle_headers(protocol, event)
        protocol._sessions[7].close.assert_called_once_with()
        protocol._handle_websocket_connect.assert_called_once_with(event)

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_connect_headers_ended_before_handshake_deliver_eof(self):
        async def run():
            protocol = _make_protocol_for_websocket_connect()
            received = threading.Event()

            def transfer_data(request):
                self.assertEqual(request.connection.read(1), b'')
                received.set()

            dispatcher = SimpleNamespace(
                get_handler_suite=lambda path: object(),
                transfer_data=transfer_data)

            class Handshaker:
                def __init__(self, request, dispatcher):
                    self.request = request

                def do_handshake(self):
                    self.request.status = 200

            with mock.patch.object(websocket_h3_server.dispatch, 'Dispatcher',
                                   return_value=dispatcher), \
                 mock.patch.object(websocket_h3_server, 'WsH3Handshaker',
                                   Handshaker):
                websocket_h3_server.WebSocketH3Protocol._handle_headers(
                    protocol, _make_headers_event(
                        7, b'CONNECT', b'websocket', True))
                self.assertTrue(await asyncio.wait_for(
                    asyncio.to_thread(received.wait, 3), 4))
                for _ in range(100):
                    if 7 not in protocol._sessions:
                        break
                    await asyncio.sleep(.01)
            self.assertNotIn(7, protocol._sessions)
            self.assertEqual(protocol._http.headers[0][2], False)
            self.assertEqual(protocol._http.data[-1], (7, b'', True))

        asyncio.run(run())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_input_overflow_resets_only_affected_stream(self):
        protocol = _make_websocket_h3_protocol()
        protocol.transmit = mock.Mock()
        protocol._quic = SimpleNamespace(
            stop_stream=mock.Mock(), reset_stream=mock.Mock())
        overloaded = SimpleNamespace(feed_data=lambda data: False,
                                     abort=mock.Mock())
        other = _make_mock_websocket_session()
        protocol._sessions = {7: overloaded, 11: other}
        websocket_h3_server.WebSocketH3Protocol._handle_data(
            protocol, SimpleNamespace(
                stream_id=7, data=b'x', stream_ended=False))
        overloaded.abort.assert_called_once_with()
        protocol._quic.stop_stream.assert_called_once_with(7, 0x101)
        protocol._quic.reset_stream.assert_called_once_with(7, 0x101)
        self.assertEqual(protocol._sessions, {11: other})

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_session_backpressure_does_not_block_loop_or_close(self):
        async def run():
            protocol = _FakeProtocol()
            loop = asyncio.get_running_loop()
            session = websocket_h3_session._WebSocketH3Session(
                protocol, 7, websocket_h3_server.H3Headers([]),
                SimpleNamespace(transfer_data=lambda request: None), loop)
            session._handler_thread = threading.current_thread()
            payload = b'x' * (8 * 1024 * 1024)
            try:
                start = time.monotonic()
                self.assertTrue(session.feed_data(payload))
                session.close()
                self.assertLess(time.monotonic() - start, 1)
                self.assertEqual(await asyncio.wait_for(
                    asyncio.to_thread(session.request.connection.read, len(payload)),
                    5), payload)
                self.assertEqual(await asyncio.wait_for(
                    asyncio.to_thread(session.request.connection.read, 1), 5),
                    b'')
            finally:
                session._rfile.close()

        asyncio.run(run())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_aborted_session_releases_blocked_reader(self):
        async def run():
            session = websocket_h3_session._WebSocketH3Session(
                _FakeProtocol(), 7, websocket_h3_server.H3Headers([]),
                SimpleNamespace(), asyncio.get_running_loop())
            reader = asyncio.to_thread(session.request.connection.read, 100)
            task = asyncio.create_task(reader)
            await asyncio.sleep(.01)
            self.assertTrue(session.feed_data(b'partial'))
            session.abort()
            self.assertEqual(await asyncio.wait_for(task, 2), b'')
            self.assertFalse(session.feed_data(b'after reset'))

        asyncio.run(run())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_configured_websocket_input_limit(self):
        reader = websocket_h3_session._WebSocketH3Input(
            max_unread_websocket_bytes=4)
        self.assertTrue(reader.feed_data(b'abcd'))
        self.assertFalse(reader.feed_data(b'e'))
        self.assertEqual(reader.read(4), b'abcd')
        self.assertTrue(reader.feed_data(b'e'))
        reader.close()

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_handshake_write_is_buffered_until_headers(self):
        async def run():
            protocol = _FakeProtocol()
            session = websocket_h3_session._WebSocketH3Session(
                protocol, 7, websocket_h3_server.H3Headers([]),
                SimpleNamespace(), asyncio.get_running_loop())
            try:
                await asyncio.wait_for(
                    asyncio.to_thread(session.request.connection.write, b'hello'),
                    2)
                self.assertEqual(protocol._http.data, [])
                protocol._http.send_headers(7, [(b':status', b'200')], False)
                session.request.connection.finish_handshake()
                self.assertEqual(protocol._http.data, [(7, b'hello', False)])
            finally:
                session.abort()

        asyncio.run(run())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_server_stop_releases_udp_port(self):
        async def fake_serve(host, port, **kwargs):
            loop = asyncio.get_running_loop()
            transport, _ = await loop.create_datagram_endpoint(
                asyncio.DatagramProtocol, local_addr=(host, port))
            return SimpleNamespace(
                _transport=transport, _protocols={}, close=transport.close)

        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        server = websocket_h3_server.WebSocketH3Server(
            '127.0.0.1', port, '', 'unused', 'unused')
        with mock.patch.object(websocket_h3_server.QuicConfiguration,
                               'load_cert_chain'), \
             mock.patch.object(websocket_h3_server, 'serve', fake_serve):
            server.start()
            try:
                with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
                    with self.assertRaises(OSError):
                        sock.bind(('127.0.0.1', port))
            finally:
                server.stop()
            with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
                sock.bind(('127.0.0.1', port))
        self.assertTrue(server.loop.is_closed())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_server_stop_drains_active_request_before_closing_loop(self):
        finished = threading.Event()
        registered = threading.Event()
        fake_protocol = mock.Mock()
        fake_protocol._workers = set()
        transport = mock.Mock()

        async def fake_serve(host, port, **kwargs):
            return SimpleNamespace(
                _transport=transport, _protocols={1: fake_protocol},
                close=mock.Mock())

        server = websocket_h3_server.WebSocketH3Server(
            '127.0.0.1', 0, '', 'unused', 'unused')

        async def request():
            await asyncio.sleep(.1)
            transport.close.assert_called_once_with()
            finished.set()

        with mock.patch.object(websocket_h3_server.QuicConfiguration,
                               'load_cert_chain'), \
             mock.patch.object(websocket_h3_server, 'serve', fake_serve):
            server.start()
            try:
                def schedule():
                    fake_protocol._workers.add(asyncio.create_task(request()))
                    registered.set()

                server.loop.call_soon_threadsafe(schedule)
                self.assertTrue(registered.wait(2))
            finally:
                server.stop()

        self.assertTrue(finished.is_set())
        fake_protocol._close_all_sessions.assert_called()
        self.assertTrue(server.loop.is_closed())

    @pytest.mark.skipif(not has_aioquic, reason='not having aioquic')
    def test_start_surfaces_startup_errors(self):
        """Server startup raises certificate loading errors to the caller."""
        server = websocket_h3_server.WebSocketH3Server(
            host='127.0.0.1',
            port=0,
            ws_doc_root='',
            cert_path='missing-cert.pem',
            key_path='missing-key.pem',
            logger=mock.Mock())

        with self.assertRaises(OSError):
            server.start()

        self.assertFalse(server.started)
        self.assertIsNotNone(server.server_thread)
        self.assertFalse(server.server_thread.is_alive())


if __name__ == '__main__':
    unittest.main()
