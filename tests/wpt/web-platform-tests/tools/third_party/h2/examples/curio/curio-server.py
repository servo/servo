#!/usr/bin/env python3.5
# -*- coding: utf-8 -*-
"""
curio-server.py
~~~~~~~~~~~~~~~

A fully-functional HTTP/2 server written for curio.

Requires Python 3.5+.
"""
import mimetypes
import os
import sys

from curio import Kernel, Event, spawn, socket, ssl

import h2.config
import h2.connection
import h2.events


# The maximum amount of a file we'll send in a single DATA frame.
READ_CHUNK_SIZE = 8192


def create_listening_ssl_socket(address, certfile, keyfile):
    """
    Create and return a listening TLS socket on a given address.
    """
    ssl_context = ssl.create_default_context(ssl.Purpose.CLIENT_AUTH)
    ssl_context.options |= (
        ssl.OP_NO_TLSv1 | ssl.OP_NO_TLSv1_1 | ssl.OP_NO_COMPRESSION
    )
    ssl_context.set_ciphers("ECDHE+AESGCM")
    ssl_context.load_cert_chain(certfile=certfile, keyfile=keyfile)
    ssl_context.set_alpn_protocols(["h2"])

    sock = socket.socket()
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock = ssl_context.wrap_socket(sock)
    sock.bind(address)
    sock.listen()

    return sock


async def h2_server(address, root, certfile, keyfile):
    """
    Create an HTTP/2 server at the given address.
    """
    sock = create_listening_ssl_socket(address, certfile, keyfile)
    print("Now listening on %s:%d" % address)

    async with sock:
        while True:
            client, _ = await sock.accept()
            server = H2Server(client, root)
            await spawn(server.run())


class H2Server:
    """
    A basic HTTP/2 file server. This is essentially very similar to
    SimpleHTTPServer from the standard library, but uses HTTP/2 instead of
    HTTP/1.1.
    """
    def __init__(self, sock, root):
        config = h2.config.H2Configuration(
            client_side=False, header_encoding='utf-8'
        )
        self.sock = sock
        self.conn = h2.connection.H2Connection(config=config)
        self.root = root
        self.flow_control_events = {}

    async def run(self):
        """
        Loop over the connection, managing it appropriately.
        """
        self.conn.initiate_connection()
        await self.sock.sendall(self.conn.data_to_send())

        while True:
            # 65535 is basically arbitrary here: this amounts to "give me
            # whatever data you have".
            data = await self.sock.recv(65535)
            if not data:
                break

            events = self.conn.receive_data(data)
            for event in events:
                if isinstance(event, h2.events.RequestReceived):
                    await spawn(
                        self.request_received(event.headers, event.stream_id)
                    )
                elif isinstance(event, h2.events.DataReceived):
                    self.conn.reset_stream(event.stream_id)
                elif isinstance(event, h2.events.WindowUpdated):
                    await self.window_updated(event)

            await self.sock.sendall(self.conn.data_to_send())

    async def request_received(self, headers, stream_id):
        """
        Handle a request by attempting to serve a suitable file.
        """
        headers = dict(headers)
        assert headers[':method'] == 'GET'

        path = headers[':path'].lstrip('/')
        full_path = os.path.join(self.root, path)

        if not os.path.exists(full_path):
            response_headers = (
                (':status', '404'),
                ('content-length', '0'),
                ('server', 'curio-h2'),
            )
            self.conn.send_headers(
                stream_id, response_headers, end_stream=True
            )
            await self.sock.sendall(self.conn.data_to_send())
        else:
            await self.send_file(full_path, stream_id)

    async def send_file(self, file_path, stream_id):
        """
        Send a file, obeying the rules of HTTP/2 flow control.
        """
        filesize = os.stat(file_path).st_size
        content_type, content_encoding = mimetypes.guess_type(file_path)
        response_headers = [
            (':status', '200'),
            ('content-length', str(filesize)),
            ('server', 'curio-h2'),
        ]
        if content_type:
            response_headers.append(('content-type', content_type))
        if content_encoding:
            response_headers.append(('content-encoding', content_encoding))

        self.conn.send_headers(stream_id, response_headers)
        await self.sock.sendall(self.conn.data_to_send())

        with open(file_path, 'rb', buffering=0) as f:
            await self._send_file_data(f, stream_id)

    async def _send_file_data(self, fileobj, stream_id):
        """
        Send the data portion of a file. Handles flow control rules.
        """
        while True:
            while not self.conn.local_flow_control_window(stream_id):
                await self.wait_for_flow_control(stream_id)

            chunk_size = min(
                self.conn.local_flow_control_window(stream_id),
                READ_CHUNK_SIZE,
            )

            data = fileobj.read(chunk_size)
            keep_reading = (len(data) == chunk_size)

            self.conn.send_data(stream_id, data, not keep_reading)
            await self.sock.sendall(self.conn.data_to_send())

            if not keep_reading:
                break

    async def wait_for_flow_control(self, stream_id):
        """
        Blocks until the flow control window for a given stream is opened.
        """
        evt = Event()
        self.flow_control_events[stream_id] = evt
        await evt.wait()

    async def window_updated(self, event):
        """
        Unblock streams waiting on flow control, if needed.
        """
        stream_id = event.stream_id

        if stream_id and stream_id in self.flow_control_events:
            evt = self.flow_control_events.pop(stream_id)
            await evt.set()
        elif not stream_id:
            # Need to keep a real list here to use only the events present at
            # this time.
            blocked_streams = list(self.flow_control_events.keys())
            for stream_id in blocked_streams:
                event = self.flow_control_events.pop(stream_id)
                await event.set()
        return


if __name__ == '__main__':
    host = sys.argv[2] if len(sys.argv) > 2 else "localhost"
    kernel = Kernel(with_monitor=True)
    print("Try GETting:")
    print("    On OSX after 'brew install curl --with-c-ares --with-libidn --with-nghttp2 --with-openssl':")
    print("/usr/local/opt/curl/bin/curl --tlsv1.2 --http2 -k https://localhost:5000/bundle.js")
    print("Or open a browser to: https://localhost:5000/")
    print("   (Accept all the warnings)")
    kernel.run(h2_server((host, 5000),
                         sys.argv[1],
                         "{}.crt.pem".format(host),
                         "{}.key".format(host)))
