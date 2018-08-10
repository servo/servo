# -*- coding: utf-8 -*-
"""
eventlet-server.py
~~~~~~~~~~~~~~~~~~

A fully-functional HTTP/2 server written for Eventlet.
"""
import collections
import json

import eventlet

from eventlet.green.OpenSSL import SSL, crypto
from h2.config import H2Configuration
from h2.connection import H2Connection
from h2.events import RequestReceived, DataReceived


class ConnectionManager(object):
    """
    An object that manages a single HTTP/2 connection.
    """
    def __init__(self, sock):
        config = H2Configuration(client_side=False)
        self.sock = sock
        self.conn = H2Connection(config=config)

    def run_forever(self):
        self.conn.initiate_connection()
        self.sock.sendall(self.conn.data_to_send())

        while True:
            data = self.sock.recv(65535)
            if not data:
                break

            events = self.conn.receive_data(data)

            for event in events:
                if isinstance(event, RequestReceived):
                    self.request_received(event.headers, event.stream_id)
                elif isinstance(event, DataReceived):
                    self.conn.reset_stream(event.stream_id)

            self.sock.sendall(self.conn.data_to_send())

    def request_received(self, headers, stream_id):
        headers = collections.OrderedDict(headers)
        data = json.dumps({'headers': headers}, indent=4).encode('utf-8')

        response_headers = (
            (':status', '200'),
            ('content-type', 'application/json'),
            ('content-length', len(data)),
            ('server', 'eventlet-h2'),
        )
        self.conn.send_headers(stream_id, response_headers)
        self.conn.send_data(stream_id, data, end_stream=True)


def alpn_callback(conn, protos):
    if b'h2' in protos:
        return b'h2'

    raise RuntimeError("No acceptable protocol offered!")


def npn_advertise_cb(conn):
    return [b'h2']


# Let's set up SSL. This is a lot of work in PyOpenSSL.
options = (
    SSL.OP_NO_COMPRESSION |
    SSL.OP_NO_SSLv2 |
    SSL.OP_NO_SSLv3 |
    SSL.OP_NO_TLSv1 |
    SSL.OP_NO_TLSv1_1
)
context = SSL.Context(SSL.SSLv23_METHOD)
context.set_options(options)
context.set_verify(SSL.VERIFY_NONE, lambda *args: True)
context.use_privatekey_file('server.key')
context.use_certificate_file('server.crt')
context.set_npn_advertise_callback(npn_advertise_cb)
context.set_alpn_select_callback(alpn_callback)
context.set_cipher_list(
    "ECDHE+AESGCM"
)
context.set_tmp_ecdh(crypto.get_elliptic_curve(u'prime256v1'))

server = eventlet.listen(('0.0.0.0', 443))
server = SSL.Connection(context, server)
pool = eventlet.GreenPool()

while True:
    try:
        new_sock, _ = server.accept()
        manager = ConnectionManager(new_sock)
        pool.spawn_n(manager.run_forever)
    except (SystemExit, KeyboardInterrupt):
        break
