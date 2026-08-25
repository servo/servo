# -*- coding: utf-8 -*-
"""
gevent-server.py
================

A simple HTTP/2 server written for gevent serving static files from a directory specified as input.
If no directory is provided, the current directory will be used.
"""
import mimetypes
import sys
from functools import partial
from pathlib import Path
from typing import Tuple, Dict, Optional

from gevent import socket, ssl
from gevent.event import Event
from gevent.server import StreamServer
from h2 import events
from h2.config import H2Configuration
from h2.connection import H2Connection


def get_http2_tls_context() -> ssl.SSLContext:
    ctx = ssl.create_default_context(purpose=ssl.Purpose.CLIENT_AUTH)
    ctx.options |= (
            ssl.OP_NO_SSLv2 | ssl.OP_NO_SSLv3 | ssl.OP_NO_TLSv1 | ssl.OP_NO_TLSv1_1
    )

    ctx.options |= ssl.OP_NO_COMPRESSION
    ctx.set_ciphers('ECDHE+AESGCM:ECDHE+CHACHA20:DHE+AESGCM:DHE+CHACHA20')
    ctx.load_cert_chain(certfile='localhost.crt', keyfile='localhost.key')
    ctx.set_alpn_protocols(['h2'])
    try:
        ctx.set_npn_protocols(['h2'])
    except NotImplementedError:
        pass

    return ctx


class H2Worker:

    def __init__(self, sock: socket, address: Tuple[str, str], source_dir: str = None):
        self._sock = sock
        self._address = address
        self._flow_control_events: Dict[int, Event] = {}
        self._server_name = 'gevent-h2'
        self._connection: Optional[H2Connection] = None
        self._read_chunk_size = 8192  # The maximum amount of a file we'll send in a single DATA frame

        self._check_sources_dir(source_dir)
        self._sources_dir = source_dir

        self._run()

    def _initiate_connection(self):
        config = H2Configuration(client_side=False, header_encoding='utf-8')
        self._connection = H2Connection(config=config)
        self._connection.initiate_connection()
        self._sock.sendall(self._connection.data_to_send())

    @staticmethod
    def _check_sources_dir(sources_dir: str) -> None:
        p = Path(sources_dir)
        if not p.is_dir():
            raise NotADirectoryError(f'{sources_dir} does not exists')

    def _send_error_response(self, status_code: str, event: events.RequestReceived) -> None:
        self._connection.send_headers(
            stream_id=event.stream_id,
            headers=[
                (':status', status_code),
                ('content-length', '0'),
                ('server', self._server_name),
            ],
            end_stream=True
        )
        self._sock.sendall(self._connection.data_to_send())

    def _handle_request(self, event: events.RequestReceived) -> None:
        headers = dict(event.headers)
        if headers[':method'] != 'GET':
            self._send_error_response('405', event)
            return

        file_path = Path(self._sources_dir) / headers[':path'].lstrip('/')
        if not file_path.is_file():
            self._send_error_response('404', event)
            return

        self._send_file(file_path, event.stream_id)

    def _send_file(self, file_path: Path, stream_id: int) -> None:
        """
        Send a file, obeying the rules of HTTP/2 flow control.
        """
        file_size = file_path.stat().st_size
        content_type, content_encoding = mimetypes.guess_type(str(file_path))
        response_headers = [
            (':status', '200'),
            ('content-length', str(file_size)),
            ('server', self._server_name)
        ]
        if content_type:
            response_headers.append(('content-type', content_type))
        if content_encoding:
            response_headers.append(('content-encoding', content_encoding))

        self._connection.send_headers(stream_id, response_headers)
        self._sock.sendall(self._connection.data_to_send())

        with file_path.open(mode='rb', buffering=0) as f:
            self._send_file_data(f, stream_id)

    def _send_file_data(self, file_obj, stream_id: int) -> None:
        """
        Send the data portion of a file. Handles flow control rules.
        """
        while True:
            while self._connection.local_flow_control_window(stream_id) < 1:
                self._wait_for_flow_control(stream_id)

            chunk_size = min(self._connection.local_flow_control_window(stream_id), self._read_chunk_size)
            data = file_obj.read(chunk_size)
            keep_reading = (len(data) == chunk_size)

            self._connection.send_data(stream_id, data, not keep_reading)
            self._sock.sendall(self._connection.data_to_send())

            if not keep_reading:
                break

    def _wait_for_flow_control(self, stream_id: int) -> None:
        """
        Blocks until the flow control window for a given stream is opened.
        """
        event = Event()
        self._flow_control_events[stream_id] = event
        event.wait()

    def _handle_window_update(self, event: events.WindowUpdated) -> None:
        """
        Unblock streams waiting on flow control, if needed.
        """
        stream_id = event.stream_id

        if stream_id and stream_id in self._flow_control_events:
            g_event = self._flow_control_events.pop(stream_id)
            g_event.set()
        elif not stream_id:
            # Need to keep a real list here to use only the events present at this time.
            blocked_streams = list(self._flow_control_events.keys())
            for stream_id in blocked_streams:
                g_event = self._flow_control_events.pop(stream_id)
                g_event.set()

    def _run(self) -> None:
        self._initiate_connection()

        while True:
            data = self._sock.recv(65535)
            if not data:
                break

            h2_events = self._connection.receive_data(data)
            for event in h2_events:
                if isinstance(event, events.RequestReceived):
                    self._handle_request(event)
                elif isinstance(event, events.DataReceived):
                    self._connection.reset_stream(event.stream_id)
                elif isinstance(event, events.WindowUpdated):
                    self._handle_window_update(event)

            data_to_send = self._connection.data_to_send()
            if data_to_send:
                self._sock.sendall(data_to_send)


if __name__ == '__main__':
    files_dir = sys.argv[1] if len(sys.argv) > 1 else f'{Path().cwd()}'
    server = StreamServer(('127.0.0.1', 8080), partial(H2Worker, source_dir=files_dir),
                          ssl_context=get_http2_tls_context())
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.close()
