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


import base64
import errno
import logging
import os
import random
import re
import socket
import struct
import time

from mod_pywebsocket import common
from mod_pywebsocket import util


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
_UPGRADE_HEADER_HIXIE75 = 'Upgrade: WebSocket\r\n'
_CONNECTION_HEADER = 'Connection: Upgrade\r\n'

WEBSOCKET_ACCEPT_UUID = '258EAFA5-E914-47DA-95CA-C5AB0DC85B11'

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
_DEFLATE_FRAME_EXTENSION = 'deflate-frame'
# TODO(bashi): Update after mux implementation finished.
_MUX_EXTENSION = 'mux_DO_NOT_USE'
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
    if ((not secure and port != DEFAULT_PORT) or
        (secure and port != DEFAULT_SECURE_PORT)):
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
    return ''.join(received_bytes)


# TODO(tyoshino): Now the WebSocketHandshake class diverts these methods. We
# should move to HTTP parser as specified in RFC 6455. For HyBi 00 and
# Hixie 75, pack these methods as some parser class.


def _read_fields(socket):
    # 4.1 32. let /fields/ be a list of name-value pairs, initially empty.
    fields = {}
    while True:
        # 4.1 33. let /name/ and /value/ be empty byte arrays
        name = ''
        value = ''
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
        if ch != '\n':  # 0x0A
            raise Exception(
                'Expected LF but found %r while reading value %r for header '
                '%r' % (ch, name, value))
        # 4.1 38. append an entry to the /fields/ list that has the name
        # given by the string obtained by interpreting the /name/ byte
        # array as a UTF-8 stream and the value given by the string
        # obtained by interpreting the /value/ byte array as a UTF-8 byte
        # stream.
        fields.setdefault(name, []).append(value)
        # 4.1 39. return to the "Field" step above
    return fields


def _read_name(socket):
    # 4.1 33. let /name/ be empty byte arrays
    name = ''
    while True:
        # 4.1 34. read a byte from the server
        ch = receive_bytes(socket, 1)
        if ch == '\r':  # 0x0D
            return None
        elif ch == '\n':  # 0x0A
            raise Exception(
                'Unexpected LF when reading header name %r' % name)
        elif ch == ':':  # 0x3A
            return name
        elif ch >= 'A' and ch <= 'Z':  # range 0x31 to 0x5A
            ch = chr(ord(ch) + 0x20)
            name += ch
        else:
            name += ch


def _skip_spaces(socket):
    # 4.1 35. read a byte from the server
    while True:
        ch = receive_bytes(socket, 1)
        if ch == ' ':  # 0x20
            continue
        return ch


def _read_value(socket, ch):
    # 4.1 33. let /value/ be empty byte arrays
    value = ''
    # 4.1 36. read a byte from server.
    while True:
        if ch == '\r':  # 0x0D
            return value
        elif ch == '\n':  # 0x0A
            raise Exception(
                'Unexpected LF when reading header value %r' % value)
        else:
            value += ch
        ch = receive_bytes(socket, 1)


def read_frame_header(socket):
    received = receive_bytes(socket, 2)

    first_byte = ord(received[0])
    fin = (first_byte >> 7) & 1
    rsv1 = (first_byte >> 6) & 1
    rsv2 = (first_byte >> 5) & 1
    rsv3 = (first_byte >> 4) & 1
    opcode = first_byte & 0xf

    second_byte = ord(received[1])
    mask = (second_byte >> 7) & 1
    payload_length = second_byte & 0x7f

    if mask != 0:
        raise Exception(
            'Mask bit must be 0 for frames coming from server')

    if payload_length == 127:
        extended_payload_length = receive_bytes(socket, 8)
        payload_length = struct.unpack(
            '!Q', extended_payload_length)[0]
        if payload_length > 0x7FFFFFFFFFFFFFFF:
            raise Exception('Extended payload length >= 2^63')
    elif payload_length == 126:
        extended_payload_length = receive_bytes(socket, 2)
        payload_length = struct.unpack(
            '!H', extended_payload_length)[0]

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
        self._socket.sendall(request_line)

        fields = []
        fields.append(_UPGRADE_HEADER)
        fields.append(_CONNECTION_HEADER)

        fields.append(_format_host_header(
            self._options.server_host,
            self._options.server_port,
            self._options.use_tls))

        if self._options.version is 8:
            fields.append(_sec_origin_header(self._options.origin))
        else:
            fields.append(_origin_header(self._options.origin))

        original_key = os.urandom(16)
        key = base64.b64encode(original_key)
        self._logger.debug(
            'Sec-WebSocket-Key: %s (%s)', key, util.hexify(original_key))
        fields.append('Sec-WebSocket-Key: %s\r\n' % key)

        fields.append('Sec-WebSocket-Version: %d\r\n' % self._options.version)

        # Setting up extensions.
        if len(self._options.extensions) > 0:
            fields.append('Sec-WebSocket-Extensions: %s\r\n' %
                          ', '.join(self._options.extensions))

        self._logger.debug('Opening handshake request headers: %r', fields)

        for field in fields:
            self._socket.sendall(field)
        self._socket.sendall('\r\n')

        self._logger.info('Sent opening handshake request')

        field = ''
        while True:
            ch = receive_bytes(self._socket, 1)
            field += ch
            if ch == '\n':
                break

        self._logger.debug('Opening handshake Response-Line: %r', field)

        if len(field) < 7 or not field.endswith('\r\n'):
            raise Exception('Wrong status line: %r' % field)
        m = re.match('[^ ]* ([^ ]*) .*', field)
        if m is None:
            raise Exception(
                'No HTTP status code found in status line: %r' % field)
        code = m.group(1)
        if not re.match('[0-9][0-9][0-9]', code):
            raise Exception(
                'HTTP status code %r is not three digit in status line: %r' %
                (code, field))
        if code != '101':
            raise HttpStatusException(
                'Expected HTTP status code 101 but found %r in status line: '
                '%r' % (code, field), int(code))
        fields = _read_fields(self._socket)
        ch = receive_bytes(self._socket, 1)
        if ch != '\n':  # 0x0A
            raise Exception('Expected LF but found: %r' % ch)

        self._logger.debug('Opening handshake response headers: %r', fields)

        # Check /fields/
        if len(fields['upgrade']) != 1:
            raise Exception(
                'Multiple Upgrade headers found: %s' % fields['upgrade'])
        if len(fields['connection']) != 1:
            raise Exception(
                'Multiple Connection headers found: %s' % fields['connection'])
        if fields['upgrade'][0] != 'websocket':
            raise Exception(
                'Unexpected Upgrade header value: %s' % fields['upgrade'][0])
        if fields['connection'][0].lower() != 'upgrade':
            raise Exception(
                'Unexpected Connection header value: %s' %
                fields['connection'][0])

        if len(fields['sec-websocket-accept']) != 1:
            raise Exception(
                'Multiple Sec-WebSocket-Accept headers found: %s' %
                fields['sec-websocket-accept'])

        accept = fields['sec-websocket-accept'][0]

        # Validate
        try:
            decoded_accept = base64.b64decode(accept)
        except TypeError, e:
            raise HandshakeException(
                'Illegal value for header Sec-WebSocket-Accept: ' + accept)

        if len(decoded_accept) != 20:
            raise HandshakeException(
                'Decoded value of Sec-WebSocket-Accept is not 20-byte long')

        self._logger.debug('Actual Sec-WebSocket-Accept: %r (%s)',
                           accept, util.hexify(decoded_accept))

        original_expected_accept = util.sha1_hash(
            key + WEBSOCKET_ACCEPT_UUID).digest()
        expected_accept = base64.b64encode(original_expected_accept)

        self._logger.debug('Expected Sec-WebSocket-Accept: %r (%s)',
                           expected_accept,
                           util.hexify(original_expected_accept))

        if accept != expected_accept:
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
        deflate_frame_accepted = False
        mux_accepted = False
        for extension in accepted_extensions:
            if extension.name() == _DEFLATE_FRAME_EXTENSION:
                if self._options.use_deflate_frame:
                    deflate_frame_accepted = True
                    continue
            if extension.name() == _MUX_EXTENSION:
                if self._options.use_mux:
                    mux_accepted = True
                    continue
            if extension.name() == _PERMESSAGE_DEFLATE_EXTENSION:
                checker = self._options.check_permessage_deflate
                if checker:
                    checker(extension)
                    continue

            raise Exception(
                'Received unrecognized extension: %s' % extension.name())

        # Let all extensions check the response for extension request.

        if (self._options.use_deflate_frame and
            not deflate_frame_accepted):
            raise Exception('%s extension not accepted' %
                            _DEFLATE_FRAME_EXTENSION)

        if self._options.use_mux and not mux_accepted:
            raise Exception('%s extension not accepted' % _MUX_EXTENSION)


class WebSocketHybi00Handshake(object):
    """Opening handshake processor for the WebSocket protocol version HyBi 00.
    """

    def __init__(self, options, draft_field):
        self._logger = util.get_class_logger(self)

        self._options = options
        self._draft_field = draft_field

    def handshake(self, socket):
        """Handshake WebSocket.

        Raises:
            Exception: handshake failed.
        """

        self._socket = socket

        # 4.1 5. send request line.
        request_line = _method_line(self._options.resource)
        self._logger.debug('Opening handshake Request-Line: %r', request_line)
        self._socket.sendall(request_line)
        # 4.1 6. Let /fields/ be an empty list of strings.
        fields = []
        # 4.1 7. Add the string "Upgrade: WebSocket" to /fields/.
        fields.append(_UPGRADE_HEADER_HIXIE75)
        # 4.1 8. Add the string "Connection: Upgrade" to /fields/.
        fields.append(_CONNECTION_HEADER)
        # 4.1 9-12. Add Host: field to /fields/.
        fields.append(_format_host_header(
            self._options.server_host,
            self._options.server_port,
            self._options.use_tls))
        # 4.1 13. Add Origin: field to /fields/.
        fields.append(_origin_header(self._options.origin))
        # TODO: 4.1 14 Add Sec-WebSocket-Protocol: field to /fields/.
        # TODO: 4.1 15 Add cookie headers to /fields/.

        # 4.1 16-23. Add Sec-WebSocket-Key<n> to /fields/.
        self._number1, key1 = self._generate_sec_websocket_key()
        self._logger.debug('Number1: %d', self._number1)
        fields.append('Sec-WebSocket-Key1: %s\r\n' % key1)
        self._number2, key2 = self._generate_sec_websocket_key()
        self._logger.debug('Number2: %d', self._number1)
        fields.append('Sec-WebSocket-Key2: %s\r\n' % key2)

        fields.append('Sec-WebSocket-Draft: %s\r\n' % self._draft_field)

        # 4.1 24. For each string in /fields/, in a random order: send the
        # string, encoded as UTF-8, followed by a UTF-8 encoded U+000D CARRIAGE
        # RETURN U+000A LINE FEED character pair (CRLF).
        random.shuffle(fields)

        self._logger.debug('Opening handshake request headers: %r', fields)
        for field in fields:
            self._socket.sendall(field)

        # 4.1 25. send a UTF-8-encoded U+000D CARRIAGE RETURN U+000A LINE FEED
        # character pair (CRLF).
        self._socket.sendall('\r\n')
        # 4.1 26. let /key3/ be a string consisting of eight random bytes (or
        # equivalently, a random 64 bit integer encoded in a big-endian order).
        self._key3 = self._generate_key3()
        # 4.1 27. send /key3/ to the server.
        self._socket.sendall(self._key3)
        self._logger.debug(
            'Key3: %r (%s)', self._key3, util.hexify(self._key3))

        self._logger.info('Sent opening handshake request')

        # 4.1 28. Read bytes from the server until either the connection
        # closes, or a 0x0A byte is read. let /field/ be these bytes, including
        # the 0x0A bytes.
        field = ''
        while True:
            ch = receive_bytes(self._socket, 1)
            field += ch
            if ch == '\n':
                break

        self._logger.debug('Opening handshake Response-Line: %r', field)

        # if /field/ is not at least seven bytes long, or if the last
        # two bytes aren't 0x0D and 0x0A respectively, or if it does not
        # contain at least two 0x20 bytes, then fail the WebSocket connection
        # and abort these steps.
        if len(field) < 7 or not field.endswith('\r\n'):
            raise Exception('Wrong status line: %r' % field)
        m = re.match('[^ ]* ([^ ]*) .*', field)
        if m is None:
            raise Exception('No code found in status line: %r' % field)
        # 4.1 29. let /code/ be the substring of /field/ that starts from the
        # byte after the first 0x20 byte, and ends with the byte before the
        # second 0x20 byte.
        code = m.group(1)
        # 4.1 30. if /code/ is not three bytes long, or if any of the bytes in
        # /code/ are not in the range 0x30 to 0x90, then fail the WebSocket
        # connection and abort these steps.
        if not re.match('[0-9][0-9][0-9]', code):
            raise Exception(
                'HTTP status code %r is not three digit in status line: %r' %
                (code, field))
        # 4.1 31. if /code/, interpreted as UTF-8, is "101", then move to the
        # next step.
        if code != '101':
            raise HttpStatusException(
                'Expected HTTP status code 101 but found %r in status line: '
                '%r' % (code, field), int(code))
        # 4.1 32-39. read fields into /fields/
        fields = _read_fields(self._socket)

        self._logger.debug('Opening handshake response headers: %r', fields)

        # 4.1 40. _Fields processing_
        # read a byte from server
        ch = receive_bytes(self._socket, 1)
        if ch != '\n':  # 0x0A
            raise Exception('Expected LF but found %r' % ch)
        # 4.1 41. check /fields/
        if len(fields['upgrade']) != 1:
            raise Exception(
                'Multiple Upgrade headers found: %s' % fields['upgrade'])
        if len(fields['connection']) != 1:
            raise Exception(
                'Multiple Connection headers found: %s' % fields['connection'])
        if len(fields['sec-websocket-origin']) != 1:
            raise Exception(
                'Multiple Sec-WebSocket-Origin headers found: %s' %
                fields['sec-sebsocket-origin'])
        if len(fields['sec-websocket-location']) != 1:
            raise Exception(
                'Multiple Sec-WebSocket-Location headers found: %s' %
                fields['sec-sebsocket-location'])
        # TODO(ukai): protocol
        # if the entry's name is "upgrade"
        #  if the value is not exactly equal to the string "WebSocket",
        #  then fail the WebSocket connection and abort these steps.
        if fields['upgrade'][0] != 'WebSocket':
            raise Exception(
                'Unexpected Upgrade header value: %s' % fields['upgrade'][0])
        # if the entry's name is "connection"
        #  if the value, converted to ASCII lowercase, is not exactly equal
        #  to the string "upgrade", then fail the WebSocket connection and
        #  abort these steps.
        if fields['connection'][0].lower() != 'upgrade':
            raise Exception(
                'Unexpected Connection header value: %s' %
                fields['connection'][0])
        # TODO(ukai): check origin, location, cookie, ..

        # 4.1 42. let /challenge/ be the concatenation of /number_1/,
        # expressed as a big endian 32 bit integer, /number_2/, expressed
        # as big endian 32 bit integer, and the eight bytes of /key_3/ in the
        # order they were sent on the wire.
        challenge = struct.pack('!I', self._number1)
        challenge += struct.pack('!I', self._number2)
        challenge += self._key3

        self._logger.debug(
            'Challenge: %r (%s)', challenge, util.hexify(challenge))

        # 4.1 43. let /expected/ be the MD5 fingerprint of /challenge/ as a
        # big-endian 128 bit string.
        expected = util.md5_hash(challenge).digest()
        self._logger.debug(
            'Expected challenge response: %r (%s)',
            expected, util.hexify(expected))

        # 4.1 44. read sixteen bytes from the server.
        # let /reply/ be those bytes.
        reply = receive_bytes(self._socket, 16)
        self._logger.debug(
            'Actual challenge response: %r (%s)', reply, util.hexify(reply))

        # 4.1 45. if /reply/ does not exactly equal /expected/, then fail
        # the WebSocket connection and abort these steps.
        if expected != reply:
            raise Exception(
                'Bad challenge response: %r (expected) != %r (actual)' %
                (expected, reply))
        # 4.1 46. The *WebSocket connection is established*.

    def _generate_sec_websocket_key(self):
        # 4.1 16. let /spaces_n/ be a random integer from 1 to 12 inclusive.
        spaces = random.randint(1, 12)
        # 4.1 17. let /max_n/ be the largest integer not greater than
        #  4,294,967,295 divided by /spaces_n/.
        maxnum = 4294967295 / spaces
        # 4.1 18. let /number_n/ be a random integer from 0 to /max_n/
        # inclusive.
        number = random.randint(0, maxnum)
        # 4.1 19. let /product_n/ be the result of multiplying /number_n/ and
        # /spaces_n/ together.
        product = number * spaces
        # 4.1 20. let /key_n/ be a string consisting of /product_n/, expressed
        # in base ten using the numerals in the range U+0030 DIGIT ZERO (0) to
        # U+0039 DIGIT NINE (9).
        key = str(product)
        # 4.1 21. insert between one and twelve random characters from the
        # range U+0021 to U+002F and U+003A to U+007E into /key_n/ at random
        # positions.
        available_chars = range(0x21, 0x2f + 1) + range(0x3a, 0x7e + 1)
        n = random.randint(1, 12)
        for _ in xrange(n):
            ch = random.choice(available_chars)
            pos = random.randint(0, len(key))
            key = key[0:pos] + chr(ch) + key[pos:]
        # 4.1 22. insert /spaces_n/ U+0020 SPACE characters into /key_n/ at
        # random positions other than start or end of the string.
        for _ in xrange(spaces):
            pos = random.randint(1, len(key) - 1)
            key = key[0:pos] + ' ' + key[pos:]
        return number, key

    def _generate_key3(self):
        # 4.1 26. let /key3/ be a string consisting of eight random bytes (or
        # equivalently, a random 64 bit integer encoded in a big-endian order).
        return ''.join([chr(random.randint(0, 255)) for _ in xrange(8)])


class WebSocketHixie75Handshake(object):
    """WebSocket handshake processor for IETF Hixie 75."""

    _EXPECTED_RESPONSE = (
        'HTTP/1.1 101 Web Socket Protocol Handshake\r\n' +
        _UPGRADE_HEADER_HIXIE75 +
        _CONNECTION_HEADER)

    def __init__(self, options):
        self._logger = util.get_class_logger(self)

        self._options = options

    def _skip_headers(self):
        terminator = '\r\n\r\n'
        pos = 0
        while pos < len(terminator):
            received = receive_bytes(self._socket, 1)
            if received == terminator[pos]:
                pos += 1
            elif received == terminator[0]:
                pos = 1
            else:
                pos = 0

    def handshake(self, socket):
        self._socket = socket

        request_line = _method_line(self._options.resource)
        self._logger.debug('Opening handshake Request-Line: %r', request_line)
        self._socket.sendall(request_line)

        headers = _UPGRADE_HEADER_HIXIE75 + _CONNECTION_HEADER
        headers += _format_host_header(
            self._options.server_host,
            self._options.server_port,
            self._options.use_tls)
        headers += _origin_header(self._options.origin)
        self._logger.debug('Opening handshake request headers: %r', headers)
        self._socket.sendall(headers)

        self._socket.sendall('\r\n')

        self._logger.info('Sent opening handshake request')

        for expected_char in WebSocketHixie75Handshake._EXPECTED_RESPONSE:
            received = receive_bytes(self._socket, 1)
            if expected_char != received:
                raise Exception('Handshake failure')
        # We cut corners and skip other headers.
        self._skip_headers()


class WebSocketStream(object):
    """Frame processor for the WebSocket protocol (RFC 6455)."""

    def __init__(self, socket, handshake):
        self._handshake = handshake
        self._socket = socket

        # Filters applied to application data part of data frames.
        self._outgoing_frame_filter = None
        self._incoming_frame_filter = None

        if self._handshake._options.use_deflate_frame:
            self._outgoing_frame_filter = (
                util._RFC1979Deflater(None, False))
            self._incoming_frame_filter = util._RFC1979Inflater()

        self._fragmented = False

    def _mask_hybi(self, s):
        # TODO(tyoshino): os.urandom does open/read/close for every call. If
        # performance matters, change this to some library call that generates
        # cryptographically secure pseudo random number sequence.
        masking_nonce = os.urandom(4)
        result = [masking_nonce]
        count = 0
        for c in s:
            result.append(chr(ord(c) ^ ord(masking_nonce[count])))
            count = (count + 1) % len(masking_nonce)
        return ''.join(result)

    def send_frame_of_arbitrary_bytes(self, header, body):
        self._socket.sendall(header + self._mask_hybi(body))

    def send_data(self, payload, frame_type, end=True, mask=True,
                  rsv1=0, rsv2=0, rsv3=0):
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

        if self._handshake._options.use_deflate_frame:
            rsv1 = 1

        if mask:
            mask_bit = 1 << 7
        else:
            mask_bit = 0

        header = chr(fin << 7 | rsv1 << 6 | rsv2 << 5 | rsv3 << 4 | opcode)
        payload_length = len(payload)
        if payload_length <= 125:
            header += chr(mask_bit | payload_length)
        elif payload_length < 1 << 16:
            header += chr(mask_bit | 126) + struct.pack('!H', payload_length)
        elif payload_length < 1 << 63:
            header += chr(mask_bit | 127) + struct.pack('!Q', payload_length)
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
            raise Exception(
                'Unexpected opcode: %d (expected) vs %d (actual)' %
                (opcode, actual_opcode))

        if actual_fin != fin:
            raise Exception(
                'Unexpected fin: %d (expected) vs %d (actual)' %
                (fin, actual_fin))

        if rsv1 is None:
            rsv1 = 0
            if self._handshake._options.use_deflate_frame:
                rsv1 = 1

        if rsv2 is None:
            rsv2 = 0

        if rsv3 is None:
            rsv3 = 0

        if actual_rsv1 != rsv1:
            raise Exception(
                'Unexpected rsv1: %r (expected) vs %r (actual)' %
                (rsv1, actual_rsv1))

        if actual_rsv2 != rsv2:
            raise Exception(
                'Unexpected rsv2: %r (expected) vs %r (actual)' %
                (rsv2, actual_rsv2))

        if actual_rsv3 != rsv3:
            raise Exception(
                'Unexpected rsv3: %r (expected) vs %r (actual)' %
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

    def assert_receive_binary(self, payload, opcode=OPCODE_BINARY, fin=1,
                              rsv1=None, rsv2=None, rsv3=None):
        self._assert_receive_data(payload, opcode, fin, rsv1, rsv2, rsv3)

    def assert_receive_text(self, payload, opcode=OPCODE_TEXT, fin=1,
                            rsv1=None, rsv2=None, rsv3=None):
        self._assert_receive_data(payload.encode('utf-8'), opcode, fin, rsv1,
                                  rsv2, rsv3)

    def _build_close_frame(self, code, reason, mask):
        frame = chr(1 << 7 | OPCODE_CLOSE)

        if code is not None:
            body = struct.pack('!H', code) + reason.encode('utf-8')
        else:
            body = ''
        if mask:
            frame += chr(1 << 7 | len(body)) + self._mask_hybi(body)
        else:
            frame += chr(len(body)) + body
        return frame

    def send_close(self, code, reason):
        self._socket.sendall(
            self._build_close_frame(code, reason, True))

    def assert_receive_close(self, code, reason):
        expected_frame = self._build_close_frame(code, reason, False)
        actual_frame = receive_bytes(self._socket, len(expected_frame))
        if actual_frame != expected_frame:
            raise Exception(
                'Unexpected close frame: %r (expected) vs %r (actual)' %
                (expected_frame, actual_frame))


class WebSocketStreamHixie75(object):
    """Frame processor for the WebSocket protocol version Hixie 75 and HyBi 00.
    """

    _CLOSE_FRAME = '\xff\x00'

    def __init__(self, socket, unused_handshake):
        self._socket = socket

    def send_frame_of_arbitrary_bytes(self, header, body):
        self._socket.sendall(header + body)

    def send_data(self, payload, unused_frame_typem, unused_end, unused_mask):
        frame = ''.join(['\x00', payload, '\xff'])
        self._socket.sendall(frame)

    def send_binary(self, unused_payload, unused_end, unused_mask):
        pass

    def send_text(self, payload, unused_end, unused_mask):
        encoded_payload = payload.encode('utf-8')
        frame = ''.join(['\x00', encoded_payload, '\xff'])
        self._socket.sendall(frame)

    def assert_receive_binary(self, payload, opcode=OPCODE_BINARY, fin=1,
                              rsv1=0, rsv2=0, rsv3=0):
        raise Exception('Binary frame is not supported in hixie75')

    def assert_receive_text(self, payload):
        received = receive_bytes(self._socket, 1)

        if received != '\x00':
            raise Exception(
                'Unexpected frame type: %d (expected) vs %d (actual)' %
                (0, ord(received)))

        received = receive_bytes(self._socket, len(payload) + 1)
        if received[-1] != '\xff':
            raise Exception(
                'Termination expected: 0xff (expected) vs %r (actual)' %
                received)

        if received[0:-1] != payload:
            raise Exception(
                'Unexpected payload: %r (expected) vs %r (actual)' %
                (payload, received[0:-1]))

    def send_close(self, code, reason):
        self._socket.sendall(self._CLOSE_FRAME)

    def assert_receive_close(self, unused_code, unused_reason):
        closing = receive_bytes(self._socket, len(self._CLOSE_FRAME))
        if closing != self._CLOSE_FRAME:
            raise Exception('Didn\'t receive closing handshake')


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
        self.extensions = []
        # Enable deflate-application-data.
        self.use_deflate_frame = False
        # Enable mux
        self.use_mux = False

    def enable_deflate_frame(self):
        self.use_deflate_frame = True
        self.extensions.append(_DEFLATE_FRAME_EXTENSION)

    def enable_mux(self):
        self.use_mux = True
        self.extensions.append(_MUX_EXTENSION)


def connect_socket_with_retry(host, port, timeout, use_tls,
                              retry=10, sleep_sec=0.1):
    retry_count = 0
    while retry_count < retry:
        try:
            s = socket.socket()
            s.settimeout(timeout)
            s.connect((host, port))
            if use_tls:
                return _TLSSocket(s)
            return s
        except socket.error, e:
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
        self._socket = connect_socket_with_retry(
                self._options.server_host,
                self._options.server_port,
                self._options.socket_timeout,
                self._options.use_tls)

        self._handshake.handshake(self._socket)

        self._stream = self._stream_class(self._socket, self._handshake)

        self._logger.info('Connection established')

    def send_frame_of_arbitrary_bytes(self, header, body):
        self._stream.send_frame_of_arbitrary_bytes(header, body)

    def send_message(self, message, end=True, binary=False, raw=False,
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
        except Exception, e:
            if str(e).find(
                'Connection closed before receiving requested length ') == 0:
                return
            try:
                error_number, message = e
                for error_name in ['ECONNRESET', 'WSAECONNRESET']:
                    if (error_name in dir(errno) and
                        error_number == getattr(errno, error_name)):
                        return
            except:
                raise e
            raise e

        raise Exception('Connection is not closed (Read: %r)' % read_data)


def create_client(options):
    return Client(
        options, WebSocketHandshake(options), WebSocketStream)


def create_client_hybi00(options):
    return Client(
        options,
        WebSocketHybi00Handshake(options, '0'),
        WebSocketStreamHixie75)


def create_client_hixie75(options):
    return Client(
        options, WebSocketHixie75Handshake(options), WebSocketStreamHixie75)


# vi:sts=4 sw=4 et
