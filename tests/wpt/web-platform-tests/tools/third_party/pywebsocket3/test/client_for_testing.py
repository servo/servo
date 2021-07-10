#!/usr/bin/env python
#
# Copyright 2012, Google Inc.
# All rights reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are
# met:
#
#     * Redistributions of source code must retain the above copyright
# notice, this list of conditions and the following disclaimer.
#     * Redistributions in binary form must reproduce the above
# copyright notice, this list of conditions and the following disclaimer
# in the documentation and/or other materials provided with the
# distribution.
#     * Neither the name of Google Inc. nor the names of its
# contributors may be used to endorse or promote products derived from
# this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
# A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
# OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
# LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
# DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
# THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
# (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
"""WebSocket client utility for testing.

This module contains helper methods for performing handshake, frame
sending/receiving as a WebSocket client.

This is code for testing mod_pywebsocket. Keep this code independent from
mod_pywebsocket. Don't import e.g. Stream class for generating frame for
testing. Using util.hexify, etc. that are not related to protocol processing
is allowed.

Note:
This code is far from robust, e.g., we cut corners in handshake.
"""

from __future__ import absolute_import
import base64
import errno
import logging
import os
import random
import re
import socket
import struct
import time
from hashlib import sha1
from six import iterbytes
from six import indexbytes

from mod_pywebsocket import common
from mod_pywebsocket import util
from mod_pywebsocket.handshake import HandshakeException

DEFAULT_PORT = 80
DEFAULT_SECURE_PORT = 443

# Opcodes introduced in IETF HyBi 01 for the new framing format
OPCODE_CONTINUATION = 0x0
OPCODE_CLOSE = 0x8
OPCODE_PING = 0x9
OPCODE_PONG = 0xa
OPCODE_TEXT = 0x1
OPCODE_BINARY = 0x2

# Strings used for handshake
_UPGRADE_HEADER = 'Upgrade: websocket\r\n'
_CONNECTION_HEADER = 'Connection: Upgrade\r\n'

WEBSOCKET_ACCEPT_UUID = b'258EAFA5-E914-47DA-95CA-C5AB0DC85B11'

# Status codes
STATUS_NORMAL_CLOSURE = 1000
STATUS_GOING_AWAY = 1001
STATUS_PROTOCOL_ERROR = 1002
STATUS_UNSUPPORTED_DATA = 1003
STATUS_NO_STATUS_RECEIVED = 1005
STATUS_ABNORMAL_CLOSURE = 1006
STATUS_INVALID_FRAME_PAYLOAD_DATA = 1007
STATUS_POLICY_VIOLATION = 1008
STATUS_MESSAGE_TOO_BIG = 1009
STATUS_MANDATORY_EXT = 1010
STATUS_INTERNAL_ENDPOINT_ERROR = 1011
STATUS_TLS_HANDSHAKE = 1015

# Extension tokens
_PERMESSAGE_DEFLATE_EXTENSION = 'permessage-deflate'


def _method_line(resource):
    return 'GET %s HTTP/1.1\r\n' % resource


def _sec_origin_header(origin):
    return 'Sec-WebSocket-Origin: %s\r\n' % origin.lower()


def _origin_header(origin):
    # 4.1 13. concatenation of the string "Origin:", a U+0020 SPACE character,
    # and the /origin/ value, converted to ASCII lowercase, to /fields/.
    return 'Origin: %s\r\n' % origin.lower()


def _format_host_header(host, port, secure):
    # 4.1 9. Let /hostport/ be an empty string.
    # 4.1 10. Append the /host/ value, converted to ASCII lowercase, to
    # /hostport/
    hostport = host.lower()
    # 4.1 11. If /secure/ is false, and /port/ is not 80, or if /secure/
    # is true, and /port/ is not 443, then append a U+003A COLON character
    # (:) followed by the value of /port/, expressed as a base-ten integer,
    # to /hostport/
    if ((not secure and port != DEFAULT_PORT)
            or (secure and port != DEFAULT_SECURE_PORT)):
        hostport += ':' + str(port)
    # 4.1 12. concatenation of the string "Host:", a U+0020 SPACE
    # character, and /hostport/, to /fields/.
    return 'Host: %s\r\n' % hostport


# TODO(tyoshino): Define a base class and move these shared methods to that.


def receive_bytes(socket, length):
    received_bytes = []
    remaining = length
    while remaining > 0:
        new_received_bytes = socket.recv(remaining)
        if not new_received_bytes:
            raise Exception(
                'Connection closed before receiving requested length '
                '(requested %d bytes but received only %d bytes)' %
                (length, length - remaining))
        received_bytes.append(new_received_bytes)
        remaining -= len(new_received_bytes)
    return b''.join(received_bytes)


# TODO(tyoshino): Now the WebSocketHandshake class diverts these methods. We
# should move to HTTP parser as specified in RFC 6455.


def _read_fields(socket):
    # 4.1 32. let /fields/ be a list of name-value pairs, initially empty.
    fields = {}
    while True:
        # 4.1 33. let /name/ and /value/ be empty byte arrays
        name = b''
        value = b''
        # 4.1 34. read /name/
        name = _read_name(socket)
        if name is None:
            break
        # 4.1 35. read spaces
        # TODO(tyoshino): Skip only one space as described in the spec.
        ch = _skip_spaces(socket)
        # 4.1 36. read /value/
        value = _read_value(socket, ch)
        # 4.1 37. read a byte from the server
        ch = receive_bytes(socket, 1)
        if ch != b'\n':  # 0x0A
            raise Exception(
                'Expected LF but found %r while reading value %r for header '
                '%r' % (ch, name, value))
        # 4.1 38. append an entry to the /fields/ list that has the name
        # given by the string obtained by interpreting the /name/ byte
        # array as a UTF-8 stream and the value given by the string
        # obtained by interpreting the /value/ byte array as a UTF-8 byte
        # stream.
        fields.setdefault(name.decode('UTF-8'),
                          []).append(value.decode('UTF-8'))
        # 4.1 39. return to the "Field" step above
    return fields


def _read_name(socket):
    # 4.1 33. let /name/ be empty byte arrays
    name = b''
    while True:
        # 4.1 34. read a byte from the server
        ch = receive_bytes(socket, 1)
        if ch == b'\r':  # 0x0D
            return None
        elif ch == b'\n':  # 0x0A
            raise Exception('Unexpected LF when reading header name %r' % name)
        elif ch == b':':  # 0x3A
            return name.lower()
        else:
            name += ch


def _skip_spaces(socket):
    # 4.1 35. read a byte from the server
    while True:
        ch = receive_bytes(socket, 1)
        if ch == b' ':  # 0x20
            continue
        return ch


def _read_value(socket, ch):
    # 4.1 33. let /value/ be empty byte arrays
    value = b''
    # 4.1 36. read a byte from server.
    while True:
        if ch == b'\r':  # 0x0D
            return value
        elif ch == b'\n':  # 0x0A
            raise Exception('Unexpected LF when reading header value %r' %
                            value)
        else:
            value += ch
        ch = receive_bytes(socket, 1)


def read_frame_header(socket):

    first_byte = ord(receive_bytes(socket, 1))
    fin = (first_byte >> 7) & 1
    rsv1 = (first_byte >> 6) & 1
    rsv2 = (first_byte >> 5) & 1
    rsv3 = (first_byte >> 4) & 1
    opcode = first_byte & 0xf

    second_byte = ord(receive_bytes(socket, 1))
    mask = (second_byte >> 7) & 1
    payload_length = second_byte & 0x7f

    if mask != 0:
        raise Exception('Mask bit must be 0 for frames coming from server')

    if payload_length == 127:
        extended_payload_length = receive_bytes(socket, 8)
        payload_length = struct.unpack('!Q', extended_payload_length)[0]
        if payload_length > 0x7FFFFFFFFFFFFFFF:
            raise Exception('Extended payload length >= 2^63')
    elif payload_length == 126:
        extended_payload_length = receive_bytes(socket, 2)
        payload_length = struct.unpack('!H', extended_payload_length)[0]

    return fin, rsv1, rsv2, rsv3, opcode, payload_length


class _TLSSocket(object):
    """Wrapper for a TLS connection."""
    def __init__(self, raw_socket):
        self._ssl = socket.ssl(raw_socket)

    def send(self, bytes):
        return self._ssl.write(bytes)

    def recv(self, size=-1):
        return self._ssl.read(size)

    def close(self):
        # Nothing to do.
        pass


class HttpStatusException(Exception):
    """This exception will be raised when unexpected http status code was
    received as a result of handshake.
    """
    def __init__(self, name, status):
        super(HttpStatusException, self).__init__(name)
        self.status = status


class WebSocketHandshake(object):
    """Opening handshake processor for the WebSocket protocol (RFC 6455)."""
    def __init__(self, options):
        self._logger = util.get_class_logger(self)

        self._options = options

    def handshake(self, socket):
        """Handshake WebSocket.

        Raises:
            Exception: handshake failed.
        """

        self._socket = socket

        request_line = _method_line(self._options.resource)
        self._logger.debug('Opening handshake Request-Line: %r', request_line)
        self._socket.sendall(request_line.encode('UTF-8'))

        fields = []
        fields.append(_UPGRADE_HEADER)
        fields.append(_CONNECTION_HEADER)

        fields.append(
            _format_host_header(self._options.server_host,
                                self._options.server_port,
                                self._options.use_tls))

        if self._options.version == 8:
            fields.append(_sec_origin_header(self._options.origin))
        else:
            fields.append(_origin_header(self._options.origin))

        original_key = os.urandom(16)
        key = base64.b64encode(original_key)
        self._logger.debug('Sec-WebSocket-Key: %s (%s)', key,
                           util.hexify(original_key))
        fields.append(u'Sec-WebSocket-Key: %s\r\n' % key.decode('UTF-8'))

        fields.append(u'Sec-WebSocket-Version: %d\r\n' % self._options.version)

        if self._options.use_basic_auth:
            credential = 'Basic ' + base64.b64encode(
                self._options.basic_auth_credential.encode('UTF-8')).decode()
            fields.append(u'Authorization: %s\r\n' % credential)

        # Setting up extensions.
        if len(self._options.extensions) > 0:
            fields.append(u'Sec-WebSocket-Extensions: %s\r\n' %
                          ', '.join(self._options.extensions))

        self._logger.debug('Opening handshake request headers: %r', fields)

        for field in fields:
            self._socket.sendall(field.encode('UTF-8'))
        self._socket.sendall(b'\r\n')

        self._logger.info('Sent opening handshake request')

        field = b''
        while True:
            ch = receive_bytes(self._socket, 1)
            field += ch
            if ch == b'\n':
                break

        self._logger.debug('Opening handshake Response-Line: %r', field)

        # Will raise a UnicodeDecodeError when the decode fails
        if len(field) < 7 or not field.endswith(b'\r\n'):
            raise Exception('Wrong status line: %s' % field.decode('Latin-1'))
        m = re.match(b'[^ ]* ([^ ]*) .*', field)
        if m is None:
            raise Exception('No HTTP status code found in status line: %s' %
                            field.decode('Latin-1'))
        code = m.group(1)
        if not re.match(b'[0-9][0-9][0-9]$', code):
            raise Exception(
                'HTTP status code %s is not three digit in status line: %s' %
                (code.decode('Latin-1'), field.decode('Latin-1')))
        if code != b'101':
            raise HttpStatusException(
                'Expected HTTP status code 101 but found %s in status line: '
                '%r' % (code.decode('Latin-1'), field.decode('Latin-1')),
                int(code))
        fields = _read_fields(self._socket)
        ch = receive_bytes(self._socket, 1)
        if ch != b'\n':  # 0x0A
            raise Exception('Expected LF but found: %r' % ch)

        self._logger.debug('Opening handshake response headers: %r', fields)

        # Check /fields/
        if len(fields['upgrade']) != 1:
            raise Exception('Multiple Upgrade headers found: %s' %
                            fields['upgrade'])
        if len(fields['connection']) != 1:
            raise Exception('Multiple Connection headers found: %s' %
                            fields['connection'])
        if fields['upgrade'][0] != 'websocket':
            raise Exception('Unexpected Upgrade header value: %s' %
                            fields['upgrade'][0])
        if fields['connection'][0].lower() != 'upgrade':
            raise Exception('Unexpected Connection header value: %s' %
                            fields['connection'][0])

        if len(fields['sec-websocket-accept']) != 1:
            raise Exception('Multiple Sec-WebSocket-Accept headers found: %s' %
                            fields['sec-websocket-accept'])

        accept = fields['sec-websocket-accept'][0]

        # Validate
        try:
            decoded_accept = base64.b64decode(accept)
        except TypeError as e:
            raise HandshakeException(
                'Illegal value for header Sec-WebSocket-Accept: ' + accept)

        if len(decoded_accept) != 20:
            raise HandshakeException(
                'Decoded value of Sec-WebSocket-Accept is not 20-byte long')

        self._logger.debug('Actual Sec-WebSocket-Accept: %r (%s)', accept,
                           util.hexify(decoded_accept))

        original_expected_accept = sha1(key + WEBSOCKET_ACCEPT_UUID).digest()
        expected_accept = base64.b64encode(original_expected_accept)

        self._logger.debug('Expected Sec-WebSocket-Accept: %r (%s)',
                           expected_accept,
                           util.hexify(original_expected_accept))

        if accept != expected_accept.decode('UTF-8'):
            raise Exception(
                'Invalid Sec-WebSocket-Accept header: %r (expected) != %r '
                '(actual)' % (accept, expected_accept))

        server_extensions_header = fields.get('sec-websocket-extensions')
        accepted_extensions = []
        if server_extensions_header is not None:
            accepted_extensions = common.parse_extensions(
                ', '.join(server_extensions_header))

        # Scan accepted extension list to check if there is any unrecognized
        # extensions or extensions we didn't request in it. Then, for
        # extensions we request, parse them and store parameters. They will be
        # used later by each extension.
        for extension in accepted_extensions:
            if extension.name() == _PERMESSAGE_DEFLATE_EXTENSION:
                checker = self._options.check_permessage_deflate
                if checker:
                    checker(extension)
                    continue

            raise Exception('Received unrecognized extension: %s' %
                            extension.name())


class WebSocketStream(object):
    """Frame processor for the WebSocket protocol (RFC 6455)."""
    def __init__(self, socket, handshake):
        self._handshake = handshake
        self._socket = socket

        # Filters applied to application data part of data frames.
        self._outgoing_frame_filter = None
        self._incoming_frame_filter = None

        self._fragmented = False

    def _mask_hybi(self, s):
        # TODO(tyoshino): os.urandom does open/read/close for every call. If
        # performance matters, change this to some library call that generates
        # cryptographically secure pseudo random number sequence.
        masking_nonce = os.urandom(4)
        result = [masking_nonce]
        count = 0
        for c in iterbytes(s):
            result.append(util.pack_byte(c ^ indexbytes(masking_nonce, count)))
            count = (count + 1) % len(masking_nonce)
        return b''.join(result)

    def send_frame_of_arbitrary_bytes(self, header, body):
        self._socket.sendall(header + self._mask_hybi(body))

    def send_data(self,
                  payload,
                  frame_type,
                  end=True,
                  mask=True,
                  rsv1=0,
                  rsv2=0,
                  rsv3=0):
        if self._outgoing_frame_filter is not None:
            payload = self._outgoing_frame_filter.filter(payload)

        if self._fragmented:
            opcode = OPCODE_CONTINUATION
        else:
            opcode = frame_type

        if end:
            self._fragmented = False
            fin = 1
        else:
            self._fragmented = True
            fin = 0

        if mask:
            mask_bit = 1 << 7
        else:
            mask_bit = 0

        header = util.pack_byte(fin << 7 | rsv1 << 6 | rsv2 << 5 | rsv3 << 4
                                | opcode)
        payload_length = len(payload)
        if payload_length <= 125:
            header += util.pack_byte(mask_bit | payload_length)
        elif payload_length < 1 << 16:
            header += util.pack_byte(mask_bit | 126) + struct.pack(
                '!H', payload_length)
        elif payload_length < 1 << 63:
            header += util.pack_byte(mask_bit | 127) + struct.pack(
                '!Q', payload_length)
        else:
            raise Exception('Too long payload (%d byte)' % payload_length)
        if mask:
            payload = self._mask_hybi(payload)
        self._socket.sendall(header + payload)

    def send_binary(self, payload, end=True, mask=True):
        self.send_data(payload, OPCODE_BINARY, end, mask)

    def send_text(self, payload, end=True, mask=True):
        self.send_data(payload.encode('utf-8'), OPCODE_TEXT, end, mask)

    def _assert_receive_data(self, payload, opcode, fin, rsv1, rsv2, rsv3):
        (actual_fin, actual_rsv1, actual_rsv2, actual_rsv3, actual_opcode,
         payload_length) = read_frame_header(self._socket)

        if actual_opcode != opcode:
            raise Exception('Unexpected opcode: %d (expected) vs %d (actual)' %
                            (opcode, actual_opcode))

        if actual_fin != fin:
            raise Exception('Unexpected fin: %d (expected) vs %d (actual)' %
                            (fin, actual_fin))

        if rsv1 is None:
            rsv1 = 0

        if rsv2 is None:
            rsv2 = 0

        if rsv3 is None:
            rsv3 = 0

        if actual_rsv1 != rsv1:
            raise Exception('Unexpected rsv1: %r (expected) vs %r (actual)' %
                            (rsv1, actual_rsv1))

        if actual_rsv2 != rsv2:
            raise Exception('Unexpected rsv2: %r (expected) vs %r (actual)' %
                            (rsv2, actual_rsv2))

        if actual_rsv3 != rsv3:
            raise Exception('Unexpected rsv3: %r (expected) vs %r (actual)' %
                            (rsv3, actual_rsv3))

        received = receive_bytes(self._socket, payload_length)

        if self._incoming_frame_filter is not None:
            received = self._incoming_frame_filter.filter(received)

        if len(received) != len(payload):
            raise Exception(
                'Unexpected payload length: %d (expected) vs %d (actual)' %
                (len(payload), len(received)))

        if payload != received:
            raise Exception(
                'Unexpected payload: %r (expected) vs %r (actual)' %
                (payload, received))

    def assert_receive_binary(self,
                              payload,
                              opcode=OPCODE_BINARY,
                              fin=1,
                              rsv1=None,
                              rsv2=None,
                              rsv3=None):
        self._assert_receive_data(payload, opcode, fin, rsv1, rsv2, rsv3)

    def assert_receive_text(self,
                            payload,
                            opcode=OPCODE_TEXT,
                            fin=1,
                            rsv1=None,
                            rsv2=None,
                            rsv3=None):
        self._assert_receive_data(payload.encode('utf-8'), opcode, fin, rsv1,
                                  rsv2, rsv3)

    def _build_close_frame(self, code, reason, mask):
        frame = util.pack_byte(1 << 7 | OPCODE_CLOSE)

        if code is not None:
            body = struct.pack('!H', code) + reason.encode('utf-8')
        else:
            body = b''
        if mask:
            frame += util.pack_byte(1 << 7 | len(body)) + self._mask_hybi(body)
        else:
            frame += util.pack_byte(len(body)) + body
        return frame

    def send_close(self, code, reason):
        self._socket.sendall(self._build_close_frame(code, reason, True))

    def assert_receive_close(self, code, reason):
        expected_frame = self._build_close_frame(code, reason, False)
        actual_frame = receive_bytes(self._socket, len(expected_frame))
        if actual_frame != expected_frame:
            raise Exception(
                'Unexpected close frame: %r (expected) vs %r (actual)' %
                (expected_frame, actual_frame))


class ClientOptions(object):
    """Holds option values to configure the Client object."""
    def __init__(self):
        self.version = 13
        self.server_host = ''
        self.origin = ''
        self.resource = ''
        self.server_port = -1
        self.socket_timeout = 1000
        self.use_tls = False
        self.use_basic_auth = False
        self.basic_auth_credential = 'test:test'
        self.extensions = []


def connect_socket_with_retry(host,
                              port,
                              timeout,
                              use_tls,
                              retry=10,
                              sleep_sec=0.1):
    retry_count = 0
    while retry_count < retry:
        try:
            s = socket.socket()
            s.settimeout(timeout)
            s.connect((host, port))
            if use_tls:
                return _TLSSocket(s)
            return s
        except socket.error as e:
            if e.errno != errno.ECONNREFUSED:
                raise
            else:
                retry_count = retry_count + 1
                time.sleep(sleep_sec)

    return None


class Client(object):
    """WebSocket client."""
    def __init__(self, options, handshake, stream_class):
        self._logger = util.get_class_logger(self)

        self._options = options
        self._socket = None

        self._handshake = handshake
        self._stream_class = stream_class

    def connect(self):
        self._socket = connect_socket_with_retry(self._options.server_host,
                                                 self._options.server_port,
                                                 self._options.socket_timeout,
                                                 self._options.use_tls)

        self._handshake.handshake(self._socket)

        self._stream = self._stream_class(self._socket, self._handshake)

        self._logger.info('Connection established')

    def send_frame_of_arbitrary_bytes(self, header, body):
        self._stream.send_frame_of_arbitrary_bytes(header, body)

    def send_message(self,
                     message,
                     end=True,
                     binary=False,
                     raw=False,
                     mask=True):
        if binary:
            self._stream.send_binary(message, end, mask)
        elif raw:
            self._stream.send_data(message, OPCODE_TEXT, end, mask)
        else:
            self._stream.send_text(message, end, mask)

    def assert_receive(self, payload, binary=False):
        if binary:
            self._stream.assert_receive_binary(payload)
        else:
            self._stream.assert_receive_text(payload)

    def send_close(self, code=STATUS_NORMAL_CLOSURE, reason=''):
        self._stream.send_close(code, reason)

    def assert_receive_close(self, code=STATUS_NORMAL_CLOSURE, reason=''):
        self._stream.assert_receive_close(code, reason)

    def close_socket(self):
        self._socket.close()

    def assert_connection_closed(self):
        try:
            read_data = receive_bytes(self._socket, 1)
        except Exception as e:
            if str(e).find(
                    'Connection closed before receiving requested length '
            ) == 0:
                return
            try:
                error_number, message = e
                for error_name in ['ECONNRESET', 'WSAECONNRESET']:
                    if (error_name in dir(errno)
                            and error_number == getattr(errno, error_name)):
                        return
            except:
                raise e
            raise e

        raise Exception('Connection is not closed (Read: %r)' % read_data)


def create_client(options):
    return Client(options, WebSocketHandshake(options), WebSocketStream)


# vi:sts=4 sw=4 et
