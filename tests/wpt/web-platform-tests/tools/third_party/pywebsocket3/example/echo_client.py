#!/usr/bin/env python
#
# Copyright 2011, Google Inc.
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
"""Simple WebSocket client named echo_client just because of historical reason.

mod_pywebsocket directory must be in PYTHONPATH.

Example Usage:

# server setup
 % cd $pywebsocket
 % PYTHONPATH=$cwd/src python ./mod_pywebsocket/standalone.py -p 8880 \
    -d $cwd/src/example

# run client
 % PYTHONPATH=$cwd/src python ./src/example/echo_client.py -p 8880 \
     -s localhost \
     -o http://localhost -r /echo -m test
"""

from __future__ import absolute_import
from __future__ import print_function
import base64
import codecs
from hashlib import sha1
import logging
from optparse import OptionParser
import os
import random
import re
import six
import socket
import ssl
import struct
import sys

from mod_pywebsocket import common
from mod_pywebsocket.extensions import PerMessageDeflateExtensionProcessor
from mod_pywebsocket.extensions import _PerMessageDeflateFramer
from mod_pywebsocket.extensions import _parse_window_bits
from mod_pywebsocket.stream import Stream
from mod_pywebsocket.stream import StreamOptions
from mod_pywebsocket import util

_TIMEOUT_SEC = 10
_UNDEFINED_PORT = -1

_UPGRADE_HEADER = 'Upgrade: websocket\r\n'
_CONNECTION_HEADER = 'Connection: Upgrade\r\n'

# Special message that tells the echo server to start closing handshake
_GOODBYE_MESSAGE = 'Goodbye'

_PROTOCOL_VERSION_HYBI13 = 'hybi13'


class ClientHandshakeError(Exception):
    pass


def _build_method_line(resource):
    return 'GET %s HTTP/1.1\r\n' % resource


def _origin_header(header, origin):
    # 4.1 13. concatenation of the string "Origin:", a U+0020 SPACE character,
    # and the /origin/ value, converted to ASCII lowercase, to /fields/.
    return '%s: %s\r\n' % (header, origin.lower())


def _format_host_header(host, port, secure):
    # 4.1 9. Let /hostport/ be an empty string.
    # 4.1 10. Append the /host/ value, converted to ASCII lowercase, to
    # /hostport/
    hostport = host.lower()
    # 4.1 11. If /secure/ is false, and /port/ is not 80, or if /secure/
    # is true, and /port/ is not 443, then append a U+003A COLON character
    # (:) followed by the value of /port/, expressed as a base-ten integer,
    # to /hostport/
    if ((not secure and port != common.DEFAULT_WEB_SOCKET_PORT)
            or (secure and port != common.DEFAULT_WEB_SOCKET_SECURE_PORT)):
        hostport += ':' + str(port)
    # 4.1 12. concatenation of the string "Host:", a U+0020 SPACE
    # character, and /hostport/, to /fields/.
    return '%s: %s\r\n' % (common.HOST_HEADER, hostport)


def _receive_bytes(socket, length):
    recv_bytes = []
    remaining = length
    while remaining > 0:
        received_bytes = socket.recv(remaining)
        if not received_bytes:
            raise IOError(
                'Connection closed before receiving requested length '
                '(requested %d bytes but received only %d bytes)' %
                (length, length - remaining))
        recv_bytes.append(received_bytes)
        remaining -= len(received_bytes)
    return b''.join(recv_bytes)


def _get_mandatory_header(fields, name):
    """Gets the value of the header specified by name from fields.

    This function expects that there's only one header with the specified name
    in fields. Otherwise, raises an ClientHandshakeError.
    """

    values = fields.get(name.lower())
    if values is None or len(values) == 0:
        raise ClientHandshakeError('%s header not found: %r' % (name, values))
    if len(values) > 1:
        raise ClientHandshakeError('Multiple %s headers found: %r' %
                                   (name, values))
    return values[0]


def _validate_mandatory_header(fields,
                               name,
                               expected_value,
                               case_sensitive=False):
    """Gets and validates the value of the header specified by name from
    fields.

    If expected_value is specified, compares expected value and actual value
    and raises an ClientHandshakeError on failure. You can specify case
    sensitiveness in this comparison by case_sensitive parameter. This function
    expects that there's only one header with the specified name in fields.
    Otherwise, raises an ClientHandshakeError.
    """

    value = _get_mandatory_header(fields, name)

    if ((case_sensitive and value != expected_value) or
        (not case_sensitive and value.lower() != expected_value.lower())):
        raise ClientHandshakeError(
            'Illegal value for header %s: %r (expected) vs %r (actual)' %
            (name, expected_value, value))


class _TLSSocket(object):
    """Wrapper for a TLS connection."""
    def __init__(self, raw_socket):
        self._logger = util.get_class_logger(self)

        self._tls_socket = ssl.wrap_socket(raw_socket)

        # Print cipher in use. Handshake is done on wrap_socket call.
        self._logger.info("Cipher: %s", self._tls_socket.cipher())

    def send(self, data):
        return self._tls_socket.write(data)

    def sendall(self, data):
        return self._tls_socket.sendall(data)

    def recv(self, size=-1):
        return self._tls_socket.read(size)

    def close(self):
        return self._tls_socket.close()

    def getpeername(self):
        return self._tls_socket.getpeername()


class ClientHandshakeBase(object):
    """A base class for WebSocket opening handshake processors for each
    protocol version.
    """
    def __init__(self):
        self._logger = util.get_class_logger(self)

    def _read_fields(self):
        # 4.1 32. let /fields/ be a list of name-value pairs, initially empty.
        fields = {}
        while True:  # "Field"
            # 4.1 33. let /name/ and /value/ be empty byte arrays
            name = b''
            value = b''
            # 4.1 34. read /name/
            name = self._read_name()
            if name is None:
                break
            # 4.1 35. read spaces
            # TODO(tyoshino): Skip only one space as described in the spec.
            ch = self._skip_spaces()
            # 4.1 36. read /value/
            value = self._read_value(ch)
            # 4.1 37. read a byte from the server
            ch = _receive_bytes(self._socket, 1)
            if ch != b'\n':  # 0x0A
                raise ClientHandshakeError(
                    'Expected LF but found %r while reading value %r for '
                    'header %r' % (ch, value, name))
            self._logger.debug('Received %r header', name)
            # 4.1 38. append an entry to the /fields/ list that has the name
            # given by the string obtained by interpreting the /name/ byte
            # array as a UTF-8 stream and the value given by the string
            # obtained by interpreting the /value/ byte array as a UTF-8 byte
            # stream.
            fields.setdefault(name.decode('UTF-8'),
                              []).append(value.decode('UTF-8'))
            # 4.1 39. return to the "Field" step above
        return fields

    def _read_name(self):
        # 4.1 33. let /name/ be empty byte arrays
        name = b''
        while True:
            # 4.1 34. read a byte from the server
            ch = _receive_bytes(self._socket, 1)
            if ch == b'\r':  # 0x0D
                return None
            elif ch == b'\n':  # 0x0A
                raise ClientHandshakeError(
                    'Unexpected LF when reading header name %r' % name)
            elif ch == b':':  # 0x3A
                return name.lower()
            else:
                name += ch

    def _skip_spaces(self):
        # 4.1 35. read a byte from the server
        while True:
            ch = _receive_bytes(self._socket, 1)
            if ch == b' ':  # 0x20
                continue
            return ch

    def _read_value(self, ch):
        # 4.1 33. let /value/ be empty byte arrays
        value = b''
        # 4.1 36. read a byte from server.
        while True:
            if ch == b'\r':  # 0x0D
                return value
            elif ch == b'\n':  # 0x0A
                raise ClientHandshakeError(
                    'Unexpected LF when reading header value %r' % value)
            else:
                value += ch
            ch = _receive_bytes(self._socket, 1)


def _get_permessage_deflate_framer(extension_response):
    """Validate the response and return a framer object using the parameters in
    the response. This method doesn't accept the server_.* parameters.
    """

    client_max_window_bits = None
    client_no_context_takeover = None

    client_max_window_bits_name = (
        PerMessageDeflateExtensionProcessor._CLIENT_MAX_WINDOW_BITS_PARAM)
    client_no_context_takeover_name = (
        PerMessageDeflateExtensionProcessor._CLIENT_NO_CONTEXT_TAKEOVER_PARAM)

    # We didn't send any server_.* parameter.
    # Handle those parameters as invalid if found in the response.

    for param_name, param_value in extension_response.get_parameters():
        if param_name == client_max_window_bits_name:
            if client_max_window_bits is not None:
                raise ClientHandshakeError('Multiple %s found' %
                                           client_max_window_bits_name)

            parsed_value = _parse_window_bits(param_value)
            if parsed_value is None:
                raise ClientHandshakeError(
                    'Bad %s: %r' % (client_max_window_bits_name, param_value))
            client_max_window_bits = parsed_value
        elif param_name == client_no_context_takeover_name:
            if client_no_context_takeover is not None:
                raise ClientHandshakeError('Multiple %s found' %
                                           client_no_context_takeover_name)

            if param_value is not None:
                raise ClientHandshakeError(
                    'Bad %s: Has value %r' %
                    (client_no_context_takeover_name, param_value))
            client_no_context_takeover = True

    if client_no_context_takeover is None:
        client_no_context_takeover = False

    return _PerMessageDeflateFramer(client_max_window_bits,
                                    client_no_context_takeover)


class ClientHandshakeProcessor(ClientHandshakeBase):
    """WebSocket opening handshake processor
    """
    def __init__(self, socket, options):
        super(ClientHandshakeProcessor, self).__init__()

        self._socket = socket
        self._options = options

        self._logger = util.get_class_logger(self)

    def handshake(self):
        """Performs opening handshake on the specified socket.

        Raises:
            ClientHandshakeError: handshake failed.
        """

        request_line = _build_method_line(self._options.resource)
        self._logger.debug('Client\'s opening handshake Request-Line: %r',
                           request_line)
        self._socket.sendall(request_line.encode('UTF-8'))

        fields = []
        fields.append(
            _format_host_header(self._options.server_host,
                                self._options.server_port,
                                self._options.use_tls))
        fields.append(_UPGRADE_HEADER)
        fields.append(_CONNECTION_HEADER)
        if self._options.origin is not None:
            fields.append(
                _origin_header(common.ORIGIN_HEADER, self._options.origin))

        original_key = os.urandom(16)
        self._key = base64.b64encode(original_key)
        self._logger.debug('%s: %r (%s)', common.SEC_WEBSOCKET_KEY_HEADER,
                           self._key, util.hexify(original_key))
        fields.append(
            '%s: %s\r\n' %
            (common.SEC_WEBSOCKET_KEY_HEADER, self._key.decode('UTF-8')))

        fields.append(
            '%s: %d\r\n' %
            (common.SEC_WEBSOCKET_VERSION_HEADER, common.VERSION_HYBI_LATEST))

        extensions_to_request = []

        if self._options.use_permessage_deflate:
            extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
            # Accept the client_max_window_bits extension parameter by default.
            extension.add_parameter(
                PerMessageDeflateExtensionProcessor.
                _CLIENT_MAX_WINDOW_BITS_PARAM, None)
            extensions_to_request.append(extension)

        if len(extensions_to_request) != 0:
            fields.append('%s: %s\r\n' %
                          (common.SEC_WEBSOCKET_EXTENSIONS_HEADER,
                           common.format_extensions(extensions_to_request)))

        for field in fields:
            self._socket.sendall(field.encode('UTF-8'))

        self._socket.sendall(b'\r\n')

        self._logger.debug('Sent client\'s opening handshake headers: %r',
                           fields)
        self._logger.debug('Start reading Status-Line')

        status_line = b''
        while True:
            ch = _receive_bytes(self._socket, 1)
            status_line += ch
            if ch == b'\n':
                break

        m = re.match(b'HTTP/\\d+\.\\d+ (\\d\\d\\d) .*\r\n', status_line)
        if m is None:
            raise ClientHandshakeError('Wrong status line format: %r' %
                                       status_line)
        status_code = m.group(1)
        if status_code != b'101':
            self._logger.debug(
                'Unexpected status code %s with following headers: %r',
                status_code, self._read_fields())
            raise ClientHandshakeError(
                'Expected HTTP status code 101 but found %r' % status_code)

        self._logger.debug('Received valid Status-Line')
        self._logger.debug('Start reading headers until we see an empty line')

        fields = self._read_fields()

        ch = _receive_bytes(self._socket, 1)
        if ch != b'\n':  # 0x0A
            raise ClientHandshakeError(
                'Expected LF but found %r while reading value %r for header '
                'name %r' % (ch, value, name))

        self._logger.debug('Received an empty line')
        self._logger.debug('Server\'s opening handshake headers: %r', fields)

        _validate_mandatory_header(fields, common.UPGRADE_HEADER,
                                   common.WEBSOCKET_UPGRADE_TYPE, False)

        _validate_mandatory_header(fields, common.CONNECTION_HEADER,
                                   common.UPGRADE_CONNECTION_TYPE, False)

        accept = _get_mandatory_header(fields,
                                       common.SEC_WEBSOCKET_ACCEPT_HEADER)

        # Validate
        try:
            binary_accept = base64.b64decode(accept)
        except TypeError:
            raise HandshakeError('Illegal value for header %s: %r' %
                                 (common.SEC_WEBSOCKET_ACCEPT_HEADER, accept))

        if len(binary_accept) != 20:
            raise ClientHandshakeError(
                'Decoded value of %s is not 20-byte long' %
                common.SEC_WEBSOCKET_ACCEPT_HEADER)

        self._logger.debug('Response for challenge : %r (%s)', accept,
                           util.hexify(binary_accept))

        binary_expected_accept = sha1(self._key +
                                      common.WEBSOCKET_ACCEPT_UUID).digest()
        expected_accept = base64.b64encode(binary_expected_accept)

        self._logger.debug('Expected response for challenge: %r (%s)',
                           expected_accept,
                           util.hexify(binary_expected_accept))

        if accept != expected_accept.decode('UTF-8'):
            raise ClientHandshakeError(
                'Invalid %s header: %r (expected: %s)' %
                (common.SEC_WEBSOCKET_ACCEPT_HEADER, accept, expected_accept))

        permessage_deflate_accepted = False

        extensions_header = fields.get(
            common.SEC_WEBSOCKET_EXTENSIONS_HEADER.lower())
        accepted_extensions = []
        if extensions_header is not None and len(extensions_header) != 0:
            accepted_extensions = common.parse_extensions(extensions_header[0])

        for extension in accepted_extensions:
            extension_name = extension.name()
            if (extension_name == common.PERMESSAGE_DEFLATE_EXTENSION
                    and self._options.use_permessage_deflate):
                permessage_deflate_accepted = True

                framer = _get_permessage_deflate_framer(extension)
                framer.set_compress_outgoing_enabled(True)
                self._options.use_permessage_deflate = framer
                continue

            raise ClientHandshakeError('Unexpected extension %r' %
                                       extension_name)

        if (self._options.use_permessage_deflate
                and not permessage_deflate_accepted):
            raise ClientHandshakeError(
                'Requested %s, but the server rejected it' %
                common.PERMESSAGE_DEFLATE_EXTENSION)

        # TODO(tyoshino): Handle Sec-WebSocket-Protocol
        # TODO(tyoshino): Handle Cookie, etc.


class ClientConnection(object):
    """A wrapper for socket object to provide the mp_conn interface.
    """
    def __init__(self, socket):
        self._socket = socket

    def write(self, data):
        self._socket.sendall(data)

    def read(self, n):
        return self._socket.recv(n)

    def get_remote_addr(self):
        return self._socket.getpeername()

    remote_addr = property(get_remote_addr)


class ClientRequest(object):
    """A wrapper class just to make it able to pass a socket object to
    functions that expect a mp_request object.
    """
    def __init__(self, socket):
        self._logger = util.get_class_logger(self)

        self._socket = socket
        self.connection = ClientConnection(socket)
        self.ws_version = common.VERSION_HYBI_LATEST


class EchoClient(object):
    """WebSocket echo client."""
    def __init__(self, options):
        self._options = options
        self._socket = None

        self._logger = util.get_class_logger(self)

    def run(self):
        """Run the client.

        Shake hands and then repeat sending message and receiving its echo.
        """

        self._socket = socket.socket()
        self._socket.settimeout(self._options.socket_timeout)
        try:
            self._socket.connect(
                (self._options.server_host, self._options.server_port))
            if self._options.use_tls:
                self._socket = _TLSSocket(self._socket)

            self._handshake = ClientHandshakeProcessor(self._socket,
                                                       self._options)

            self._handshake.handshake()

            self._logger.info('Connection established')

            request = ClientRequest(self._socket)

            stream_option = StreamOptions()
            stream_option.mask_send = True
            stream_option.unmask_receive = False

            if self._options.use_permessage_deflate is not False:
                framer = self._options.use_permessage_deflate
                framer.setup_stream_options(stream_option)

            self._stream = Stream(request, stream_option)

            for line in self._options.message.split(','):
                self._stream.send_message(line)
                if self._options.verbose:
                    print('Send: %s' % line)
                try:
                    received = self._stream.receive_message()

                    if self._options.verbose:
                        print('Recv: %s' % received)
                except Exception as e:
                    if self._options.verbose:
                        print('Error: %s' % e)
                    raise

            self._do_closing_handshake()
        finally:
            self._socket.close()

    def _do_closing_handshake(self):
        """Perform closing handshake using the specified closing frame."""

        if self._options.message.split(',')[-1] == _GOODBYE_MESSAGE:
            # requested server initiated closing handshake, so
            # expecting closing handshake message from server.
            self._logger.info('Wait for server-initiated closing handshake')
            message = self._stream.receive_message()
            if message is None:
                print('Recv close')
                print('Send ack')
                self._logger.info('Received closing handshake and sent ack')
                return
        print('Send close')
        self._stream.close_connection()
        self._logger.info('Sent closing handshake')
        print('Recv ack')
        self._logger.info('Received ack')


def main():
    if six.PY2:
        sys.stdout = codecs.getwriter('utf-8')(sys.stdout)

    parser = OptionParser()
    # We accept --command_line_flag style flags which is the same as Google
    # gflags in addition to common --command-line-flag style flags.
    parser.add_option('-s',
                      '--server-host',
                      '--server_host',
                      dest='server_host',
                      type='string',
                      default='localhost',
                      help='server host')
    parser.add_option('-p',
                      '--server-port',
                      '--server_port',
                      dest='server_port',
                      type='int',
                      default=_UNDEFINED_PORT,
                      help='server port')
    parser.add_option('-o',
                      '--origin',
                      dest='origin',
                      type='string',
                      default=None,
                      help='origin')
    parser.add_option('-r',
                      '--resource',
                      dest='resource',
                      type='string',
                      default='/echo',
                      help='resource path')
    parser.add_option('-m',
                      '--message',
                      dest='message',
                      type='string',
                      help=('comma-separated messages to send. '
                            '%s will force close the connection from server.' %
                            _GOODBYE_MESSAGE))
    parser.add_option('-q',
                      '--quiet',
                      dest='verbose',
                      action='store_false',
                      default=True,
                      help='suppress messages')
    parser.add_option('-t',
                      '--tls',
                      dest='use_tls',
                      action='store_true',
                      default=False,
                      help='use TLS (wss://).')
    parser.add_option('-k',
                      '--socket-timeout',
                      '--socket_timeout',
                      dest='socket_timeout',
                      type='int',
                      default=_TIMEOUT_SEC,
                      help='Timeout(sec) for sockets')
    parser.add_option('--use-permessage-deflate',
                      '--use_permessage_deflate',
                      dest='use_permessage_deflate',
                      action='store_true',
                      default=False,
                      help='Use the permessage-deflate extension.')
    parser.add_option('--log-level',
                      '--log_level',
                      type='choice',
                      dest='log_level',
                      default='warn',
                      choices=['debug', 'info', 'warn', 'error', 'critical'],
                      help='Log level.')

    (options, unused_args) = parser.parse_args()

    logging.basicConfig(level=logging.getLevelName(options.log_level.upper()))

    # Default port number depends on whether TLS is used.
    if options.server_port == _UNDEFINED_PORT:
        if options.use_tls:
            options.server_port = common.DEFAULT_WEB_SOCKET_SECURE_PORT
        else:
            options.server_port = common.DEFAULT_WEB_SOCKET_PORT

    # optparse doesn't seem to handle non-ascii default values.
    # Set default message here.
    if not options.message:
        options.message = u'Hello,\u65e5\u672c'  # "Japan" in Japanese

    EchoClient(options).run()


if __name__ == '__main__':
    main()

# vi:sts=4 sw=4 et
