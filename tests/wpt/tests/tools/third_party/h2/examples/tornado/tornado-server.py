#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""
tornado-server.py
~~~~~~~~~~~~~~~~~

A fully-functional HTTP/2 server written for Tornado.
"""
import collections
import json
import ssl

import tornado.gen
import tornado.ioloop
import tornado.iostream
import tornado.tcpserver

from h2.config import H2Configuration
from h2.connection import H2Connection
from h2.events import RequestReceived, DataReceived


def create_ssl_context(certfile, keyfile):
    ssl_context = ssl.create_default_context(ssl.Purpose.CLIENT_AUTH)
    ssl_context.options |= (
        ssl.OP_NO_TLSv1 | ssl.OP_NO_TLSv1_1 | ssl.OP_NO_COMPRESSION
    )
    ssl_context.set_ciphers("ECDHE+AESGCM")
    ssl_context.load_cert_chain(certfile=certfile, keyfile=keyfile)
    ssl_context.set_alpn_protocols(["h2"])
    return ssl_context


class H2Server(tornado.tcpserver.TCPServer):

    @tornado.gen.coroutine
    def handle_stream(self, stream, address):
        handler = EchoHeadersHandler(stream)
        yield handler.handle()


class EchoHeadersHandler(object):

    def __init__(self, stream):
        self.stream = stream

        config = H2Configuration(client_side=False)
        self.conn = H2Connection(config=config)

    @tornado.gen.coroutine
    def handle(self):
        self.conn.initiate_connection()
        yield self.stream.write(self.conn.data_to_send())

        while True:
            try:
                data = yield self.stream.read_bytes(65535, partial=True)
                if not data:
                    break

                events = self.conn.receive_data(data)
                for event in events:
                    if isinstance(event, RequestReceived):
                        self.request_received(event.headers, event.stream_id)
                    elif isinstance(event, DataReceived):
                        self.conn.reset_stream(event.stream_id)

                yield self.stream.write(self.conn.data_to_send())

            except tornado.iostream.StreamClosedError:
                break

    def request_received(self, headers, stream_id):
        headers = collections.OrderedDict(headers)
        data = json.dumps({'headers': headers}, indent=4).encode('utf-8')

        response_headers = (
            (':status', '200'),
            ('content-type', 'application/json'),
            ('content-length', str(len(data))),
            ('server', 'tornado-h2'),
        )
        self.conn.send_headers(stream_id, response_headers)
        self.conn.send_data(stream_id, data, end_stream=True)


if __name__ == '__main__':
    ssl_context = create_ssl_context('server.crt', 'server.key')
    server = H2Server(ssl_options=ssl_context)
    server.listen(8888)
    io_loop = tornado.ioloop.IOLoop.current()
    io_loop.start()
