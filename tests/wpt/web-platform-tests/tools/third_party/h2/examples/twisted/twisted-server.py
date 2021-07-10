# -*- coding: utf-8 -*-
"""
twisted-server.py
~~~~~~~~~~~~~~~~~

A fully-functional HTTP/2 server written for Twisted.
"""
import functools
import mimetypes
import os
import os.path
import sys

from OpenSSL import crypto
from twisted.internet.defer import Deferred, inlineCallbacks
from twisted.internet.protocol import Protocol, Factory
from twisted.internet import endpoints, reactor, ssl
from h2.config import H2Configuration
from h2.connection import H2Connection
from h2.events import (
    RequestReceived, DataReceived, WindowUpdated
)
from h2.exceptions import ProtocolError


def close_file(file, d):
    file.close()


READ_CHUNK_SIZE = 8192


class H2Protocol(Protocol):
    def __init__(self, root):
        config = H2Configuration(client_side=False)
        self.conn = H2Connection(config=config)
        self.known_proto = None
        self.root = root

        self._flow_control_deferreds = {}

    def connectionMade(self):
        self.conn.initiate_connection()
        self.transport.write(self.conn.data_to_send())

    def dataReceived(self, data):
        if not self.known_proto:
            self.known_proto = True

        try:
            events = self.conn.receive_data(data)
        except ProtocolError:
            if self.conn.data_to_send:
                self.transport.write(self.conn.data_to_send())
            self.transport.loseConnection()
        else:
            for event in events:
                if isinstance(event, RequestReceived):
                    self.requestReceived(event.headers, event.stream_id)
                elif isinstance(event, DataReceived):
                    self.dataFrameReceived(event.stream_id)
                elif isinstance(event, WindowUpdated):
                    self.windowUpdated(event)

            if self.conn.data_to_send:
                self.transport.write(self.conn.data_to_send())

    def requestReceived(self, headers, stream_id):
        headers = dict(headers)  # Invalid conversion, fix later.
        assert headers[b':method'] == b'GET'

        path = headers[b':path'].lstrip(b'/')
        full_path = os.path.join(self.root, path)

        if not os.path.exists(full_path):
            response_headers = (
                (':status', '404'),
                ('content-length', '0'),
                ('server', 'twisted-h2'),
            )
            self.conn.send_headers(
                stream_id, response_headers, end_stream=True
            )
            self.transport.write(self.conn.data_to_send())
        else:
            self.sendFile(full_path, stream_id)

        return

    def dataFrameReceived(self, stream_id):
        self.conn.reset_stream(stream_id)
        self.transport.write(self.conn.data_to_send())

    def sendFile(self, file_path, stream_id):
        filesize = os.stat(file_path).st_size
        content_type, content_encoding = mimetypes.guess_type(
            file_path.decode('utf-8')
        )
        response_headers = [
            (':status', '200'),
            ('content-length', str(filesize)),
            ('server', 'twisted-h2'),
        ]
        if content_type:
            response_headers.append(('content-type', content_type))
        if content_encoding:
            response_headers.append(('content-encoding', content_encoding))

        self.conn.send_headers(stream_id, response_headers)
        self.transport.write(self.conn.data_to_send())

        f = open(file_path, 'rb')
        d = self._send_file(f, stream_id)
        d.addErrback(functools.partial(close_file, f))

    def windowUpdated(self, event):
        """
        Handle a WindowUpdated event by firing any waiting data sending
        callbacks.
        """
        stream_id = event.stream_id

        if stream_id and stream_id in self._flow_control_deferreds:
            d = self._flow_control_deferreds.pop(stream_id)
            d.callback(event.delta)
        elif not stream_id:
            for d in self._flow_control_deferreds.values():
                d.callback(event.delta)

            self._flow_control_deferreds = {}

        return

    @inlineCallbacks
    def _send_file(self, file, stream_id):
        """
        This callback sends more data for a given file on the stream.
        """
        keep_reading = True
        while keep_reading:
            while not self.conn.remote_flow_control_window(stream_id):
                yield self.wait_for_flow_control(stream_id)

            chunk_size = min(
                self.conn.remote_flow_control_window(stream_id), READ_CHUNK_SIZE
            )
            data = file.read(chunk_size)
            keep_reading = len(data) == chunk_size
            self.conn.send_data(stream_id, data, not keep_reading)
            self.transport.write(self.conn.data_to_send())

            if not keep_reading:
                break

        file.close()

    def wait_for_flow_control(self, stream_id):
        """
        Returns a Deferred that fires when the flow control window is opened.
        """
        d = Deferred()
        self._flow_control_deferreds[stream_id] = d
        return d


class H2Factory(Factory):
    def __init__(self, root):
        self.root = root

    def buildProtocol(self, addr):
        print(H2Protocol)
        return H2Protocol(self.root)


root = sys.argv[1].encode('utf-8')

with open('server.crt', 'r') as f:
    cert_data = f.read()
with open('server.key', 'r') as f:
    key_data = f.read()

cert = crypto.load_certificate(crypto.FILETYPE_PEM, cert_data)
key = crypto.load_privatekey(crypto.FILETYPE_PEM, key_data)
options = ssl.CertificateOptions(
    privateKey=key,
    certificate=cert,
    acceptableProtocols=[b'h2'],
)

endpoint = endpoints.SSL4ServerEndpoint(reactor, 8080, options, backlog=128)
endpoint.listen(H2Factory(root))
reactor.run()
