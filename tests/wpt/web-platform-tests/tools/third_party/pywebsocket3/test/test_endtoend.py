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
"""End-to-end tests for pywebsocket. Tests standalone.py.
"""

from __future__ import absolute_import
from six.moves import urllib
import locale
import logging
import os
import signal
import socket
import subprocess
import sys
import time
import unittest

import set_sys_path  # Update sys.path to locate mod_pywebsocket module.

from test import client_for_testing

# Special message that tells the echo server to start closing handshake
_GOODBYE_MESSAGE = 'Goodbye'

_SERVER_WARMUP_IN_SEC = 0.2


# Test body functions
def _echo_check_procedure(client):
    client.connect()

    client.send_message('test')
    client.assert_receive('test')
    client.send_message('helloworld')
    client.assert_receive('helloworld')

    client.send_close()
    client.assert_receive_close()

    client.assert_connection_closed()


def _echo_check_procedure_with_binary(client):
    client.connect()

    client.send_message(b'binary', binary=True)
    client.assert_receive(b'binary', binary=True)
    client.send_message(b'\x00\x80\xfe\xff\x00\x80', binary=True)
    client.assert_receive(b'\x00\x80\xfe\xff\x00\x80', binary=True)

    client.send_close()
    client.assert_receive_close()

    client.assert_connection_closed()


def _echo_check_procedure_with_goodbye(client):
    client.connect()

    client.send_message('test')
    client.assert_receive('test')

    client.send_message(_GOODBYE_MESSAGE)
    client.assert_receive(_GOODBYE_MESSAGE)

    client.assert_receive_close()
    client.send_close()

    client.assert_connection_closed()


def _echo_check_procedure_with_code_and_reason(client, code, reason):
    client.connect()

    client.send_close(code, reason)
    client.assert_receive_close(code, reason)

    client.assert_connection_closed()


def _unmasked_frame_check_procedure(client):
    client.connect()

    client.send_message('test', mask=False)
    client.assert_receive_close(client_for_testing.STATUS_PROTOCOL_ERROR, '')

    client.assert_connection_closed()


def _check_handshake_with_basic_auth(client):
    client.connect()

    client.send_message(_GOODBYE_MESSAGE)
    client.assert_receive(_GOODBYE_MESSAGE)

    client.assert_receive_close()
    client.send_close()

    client.assert_connection_closed()


class EndToEndTestBase(unittest.TestCase):
    """Base class for end-to-end tests that launch pywebsocket standalone
    server as a separate process, connect to it using the client_for_testing
    module, and check if the server behaves correctly by exchanging opening
    handshake and frames over a TCP connection.
    """
    def setUp(self):
        self.server_stderr = None
        self.top_dir = os.path.join(os.path.dirname(__file__), '..')
        os.putenv('PYTHONPATH', os.path.pathsep.join(sys.path))
        self.standalone_command = os.path.join(self.top_dir, 'mod_pywebsocket',
                                               'standalone.py')
        self.document_root = os.path.join(self.top_dir, 'example')
        s = socket.socket()
        s.bind(('localhost', 0))
        (_, self.test_port) = s.getsockname()
        s.close()

        self._options = client_for_testing.ClientOptions()
        self._options.server_host = 'localhost'
        self._options.origin = 'http://localhost'
        self._options.resource = '/echo'

        self._options.server_port = self.test_port

    # TODO(tyoshino): Use tearDown to kill the server.

    def _run_python_command(self, commandline, stdout=None, stderr=None):
        close_fds = True if sys.platform != 'win32' else None
        return subprocess.Popen([sys.executable] + commandline,
                                close_fds=close_fds,
                                stdout=stdout,
                                stderr=stderr)

    def _run_server(self, extra_args=[]):
        args = [
            self.standalone_command, '-H', 'localhost', '-V', 'localhost',
            '-p',
            str(self.test_port), '-P',
            str(self.test_port), '-d', self.document_root
        ]

        # Inherit the level set to the root logger by test runner.
        root_logger = logging.getLogger()
        log_level = root_logger.getEffectiveLevel()
        if log_level != logging.NOTSET:
            args.append('--log-level')
            args.append(logging.getLevelName(log_level).lower())

        args += extra_args

        return self._run_python_command(args, stderr=self.server_stderr)

    def _close_server(self, server):
        """

        This method mimics Popen.__exit__ to gracefully kill the server process.
        Its main purpose is to maintain comptaibility between python 2 and 3,
        since Popen in python 2 does not have __exit__ attribute.

        """
        server.kill()

        if server.stdout:
            server.stdout.close()
        if server.stderr:
            server.stderr.close()
        if server.stdin:
            server.stdin.close()

        server.wait()


class EndToEndHyBiTest(EndToEndTestBase):
    def setUp(self):
        EndToEndTestBase.setUp(self)

    def _run_test_with_options(self,
                               test_function,
                               options,
                               server_options=[]):
        server = self._run_server(server_options)
        try:
            # TODO(tyoshino): add some logic to poll the server until it
            # becomes ready
            time.sleep(_SERVER_WARMUP_IN_SEC)

            client = client_for_testing.create_client(options)
            try:
                test_function(client)
            finally:
                client.close_socket()
        finally:
            self._close_server(server)

    def _run_test(self, test_function):
        self._run_test_with_options(test_function, self._options)

    def _run_permessage_deflate_test(self, offer, response_checker,
                                     test_function):
        server = self._run_server()
        try:
            time.sleep(_SERVER_WARMUP_IN_SEC)

            self._options.extensions += offer
            self._options.check_permessage_deflate = response_checker
            client = client_for_testing.create_client(self._options)

            try:
                client.connect()

                if test_function is not None:
                    test_function(client)

                client.assert_connection_closed()
            finally:
                client.close_socket()
        finally:
            self._close_server(server)

    def _run_close_with_code_and_reason_test(self,
                                             test_function,
                                             code,
                                             reason,
                                             server_options=[]):
        server = self._run_server()
        try:
            time.sleep(_SERVER_WARMUP_IN_SEC)

            client = client_for_testing.create_client(self._options)
            try:
                test_function(client, code, reason)
            finally:
                client.close_socket()
        finally:
            self._close_server(server)

    def _run_http_fallback_test(self, options, status):
        server = self._run_server()
        try:
            time.sleep(_SERVER_WARMUP_IN_SEC)

            client = client_for_testing.create_client(options)
            try:
                client.connect()
                self.fail('Could not catch HttpStatusException')
            except client_for_testing.HttpStatusException as e:
                self.assertEqual(status, e.status)
            except Exception as e:
                self.fail('Catch unexpected exception')
            finally:
                client.close_socket()
        finally:
            self._close_server(server)

    def test_echo(self):
        self._run_test(_echo_check_procedure)

    def test_echo_binary(self):
        self._run_test(_echo_check_procedure_with_binary)

    def test_echo_server_close(self):
        self._run_test(_echo_check_procedure_with_goodbye)

    def test_unmasked_frame(self):
        self._run_test(_unmasked_frame_check_procedure)

    def test_echo_permessage_deflate(self):
        def test_function(client):
            # From the examples in the spec.
            compressed_hello = b'\xf2\x48\xcd\xc9\xc9\x07\x00'
            client._stream.send_data(compressed_hello,
                                     client_for_testing.OPCODE_TEXT,
                                     rsv1=1)
            client._stream.assert_receive_binary(
                compressed_hello,
                opcode=client_for_testing.OPCODE_TEXT,
                rsv1=1)

            client.send_close()
            client.assert_receive_close()

        def response_checker(parameter):
            self.assertEqual('permessage-deflate', parameter.name())
            self.assertEqual([], parameter.get_parameters())

        self._run_permessage_deflate_test(['permessage-deflate'],
                                          response_checker, test_function)

    def test_echo_permessage_deflate_two_frames(self):
        def test_function(client):
            # From the examples in the spec.
            client._stream.send_data(b'\xf2\x48\xcd',
                                     client_for_testing.OPCODE_TEXT,
                                     end=False,
                                     rsv1=1)
            client._stream.send_data(b'\xc9\xc9\x07\x00',
                                     client_for_testing.OPCODE_TEXT)
            client._stream.assert_receive_binary(
                b'\xf2\x48\xcd\xc9\xc9\x07\x00',
                opcode=client_for_testing.OPCODE_TEXT,
                rsv1=1)

            client.send_close()
            client.assert_receive_close()

        def response_checker(parameter):
            self.assertEqual('permessage-deflate', parameter.name())
            self.assertEqual([], parameter.get_parameters())

        self._run_permessage_deflate_test(['permessage-deflate'],
                                          response_checker, test_function)

    def test_echo_permessage_deflate_two_messages(self):
        def test_function(client):
            # From the examples in the spec.
            client._stream.send_data(b'\xf2\x48\xcd\xc9\xc9\x07\x00',
                                     client_for_testing.OPCODE_TEXT,
                                     rsv1=1)
            client._stream.send_data(b'\xf2\x00\x11\x00\x00',
                                     client_for_testing.OPCODE_TEXT,
                                     rsv1=1)
            client._stream.assert_receive_binary(
                b'\xf2\x48\xcd\xc9\xc9\x07\x00',
                opcode=client_for_testing.OPCODE_TEXT,
                rsv1=1)
            client._stream.assert_receive_binary(
                b'\xf2\x00\x11\x00\x00',
                opcode=client_for_testing.OPCODE_TEXT,
                rsv1=1)

            client.send_close()
            client.assert_receive_close()

        def response_checker(parameter):
            self.assertEqual('permessage-deflate', parameter.name())
            self.assertEqual([], parameter.get_parameters())

        self._run_permessage_deflate_test(['permessage-deflate'],
                                          response_checker, test_function)

    def test_echo_permessage_deflate_two_msgs_server_no_context_takeover(self):
        def test_function(client):
            # From the examples in the spec.
            client._stream.send_data(b'\xf2\x48\xcd\xc9\xc9\x07\x00',
                                     client_for_testing.OPCODE_TEXT,
                                     rsv1=1)
            client._stream.send_data(b'\xf2\x00\x11\x00\x00',
                                     client_for_testing.OPCODE_TEXT,
                                     rsv1=1)
            client._stream.assert_receive_binary(
                b'\xf2\x48\xcd\xc9\xc9\x07\x00',
                opcode=client_for_testing.OPCODE_TEXT,
                rsv1=1)
            client._stream.assert_receive_binary(
                b'\xf2\x48\xcd\xc9\xc9\x07\x00',
                opcode=client_for_testing.OPCODE_TEXT,
                rsv1=1)

            client.send_close()
            client.assert_receive_close()

        def response_checker(parameter):
            self.assertEqual('permessage-deflate', parameter.name())
            self.assertEqual([('server_no_context_takeover', None)],
                             parameter.get_parameters())

        self._run_permessage_deflate_test(
            ['permessage-deflate; server_no_context_takeover'],
            response_checker, test_function)

    def test_echo_permessage_deflate_preference(self):
        def test_function(client):
            # From the examples in the spec.
            compressed_hello = b'\xf2\x48\xcd\xc9\xc9\x07\x00'
            client._stream.send_data(compressed_hello,
                                     client_for_testing.OPCODE_TEXT,
                                     rsv1=1)
            client._stream.assert_receive_binary(
                compressed_hello,
                opcode=client_for_testing.OPCODE_TEXT,
                rsv1=1)

            client.send_close()
            client.assert_receive_close()

        def response_checker(parameter):
            self.assertEqual('permessage-deflate', parameter.name())
            self.assertEqual([], parameter.get_parameters())

        self._run_permessage_deflate_test(
            ['permessage-deflate', 'deflate-frame'], response_checker,
            test_function)

    def test_echo_permessage_deflate_with_parameters(self):
        def test_function(client):
            # From the examples in the spec.
            compressed_hello = b'\xf2\x48\xcd\xc9\xc9\x07\x00'
            client._stream.send_data(compressed_hello,
                                     client_for_testing.OPCODE_TEXT,
                                     rsv1=1)
            client._stream.assert_receive_binary(
                compressed_hello,
                opcode=client_for_testing.OPCODE_TEXT,
                rsv1=1)

            client.send_close()
            client.assert_receive_close()

        def response_checker(parameter):
            self.assertEqual('permessage-deflate', parameter.name())
            self.assertEqual([('server_max_window_bits', '10'),
                              ('server_no_context_takeover', None)],
                             parameter.get_parameters())

        self._run_permessage_deflate_test([
            'permessage-deflate; server_max_window_bits=10; '
            'server_no_context_takeover'
        ], response_checker, test_function)

    def test_echo_permessage_deflate_with_bad_server_max_window_bits(self):
        def test_function(client):
            client.send_close()
            client.assert_receive_close()

        def response_checker(parameter):
            raise Exception('Unexpected acceptance of permessage-deflate')

        self._run_permessage_deflate_test(
            ['permessage-deflate; server_max_window_bits=3000000'],
            response_checker, test_function)

    def test_echo_permessage_deflate_with_bad_server_max_window_bits(self):
        def test_function(client):
            client.send_close()
            client.assert_receive_close()

        def response_checker(parameter):
            raise Exception('Unexpected acceptance of permessage-deflate')

        self._run_permessage_deflate_test(
            ['permessage-deflate; server_max_window_bits=3000000'],
            response_checker, test_function)

    def test_echo_permessage_deflate_with_undefined_parameter(self):
        def test_function(client):
            client.send_close()
            client.assert_receive_close()

        def response_checker(parameter):
            raise Exception('Unexpected acceptance of permessage-deflate')

        self._run_permessage_deflate_test(['permessage-deflate; foo=bar'],
                                          response_checker, test_function)

    def test_echo_close_with_code_and_reason(self):
        self._options.resource = '/close'
        self._run_close_with_code_and_reason_test(
            _echo_check_procedure_with_code_and_reason, 3333, 'sunsunsunsun')

    def test_echo_close_with_empty_body(self):
        self._options.resource = '/close'
        self._run_close_with_code_and_reason_test(
            _echo_check_procedure_with_code_and_reason, None, '')

    def test_close_on_protocol_error(self):
        """Tests that the server sends a close frame with protocol error status
        code when the client sends data with some protocol error.
        """
        def test_function(client):
            client.connect()

            # Intermediate frame without any preceding start of fragmentation
            # frame.
            client.send_frame_of_arbitrary_bytes(b'\x80\x80', '')
            client.assert_receive_close(
                client_for_testing.STATUS_PROTOCOL_ERROR)

        self._run_test(test_function)

    def test_close_on_unsupported_frame(self):
        """Tests that the server sends a close frame with unsupported operation
        status code when the client sends data asking some operation that is
        not supported by the server.
        """
        def test_function(client):
            client.connect()

            # Text frame with RSV3 bit raised.
            client.send_frame_of_arbitrary_bytes(b'\x91\x80', '')
            client.assert_receive_close(
                client_for_testing.STATUS_UNSUPPORTED_DATA)

        self._run_test(test_function)

    def test_close_on_invalid_frame(self):
        """Tests that the server sends a close frame with invalid frame payload
        data status code when the client sends an invalid frame like containing
        invalid UTF-8 character.
        """
        def test_function(client):
            client.connect()

            # Text frame with invalid UTF-8 string.
            client.send_message(b'\x80', raw=True)
            client.assert_receive_close(
                client_for_testing.STATUS_INVALID_FRAME_PAYLOAD_DATA)

        self._run_test(test_function)

    def test_close_on_internal_endpoint_error(self):
        """Tests that the server sends a close frame with internal endpoint
        error status code when the handler does bad operation.
        """

        self._options.resource = '/internal_error'

        def test_function(client):
            client.connect()
            client.assert_receive_close(
                client_for_testing.STATUS_INTERNAL_ENDPOINT_ERROR)

        self._run_test(test_function)

    def test_absolute_uri(self):
        """Tests absolute uri request."""

        options = self._options
        options.resource = 'ws://localhost:%d/echo' % options.server_port
        self._run_test_with_options(_echo_check_procedure, options)

    def test_invalid_absolute_uri(self):
        """Tests invalid absolute uri request."""

        options = self._options
        options.resource = 'ws://invalidlocalhost:%d/echo' % options.server_port
        options.server_stderr = subprocess.PIPE

        self._run_http_fallback_test(options, 404)

    def test_origin_check(self):
        """Tests http fallback on origin check fail."""

        options = self._options
        options.resource = '/origin_check'
        # Server shows warning message for http 403 fallback. This warning
        # message is confusing. Following pipe disposes warning messages.
        self.server_stderr = subprocess.PIPE
        self._run_http_fallback_test(options, 403)

    def test_invalid_resource(self):
        """Tests invalid resource path."""

        options = self._options
        options.resource = '/no_resource'

        self.server_stderr = subprocess.PIPE
        self._run_http_fallback_test(options, 404)

    def test_fragmentized_resource(self):
        """Tests resource name with fragment"""

        options = self._options
        options.resource = '/echo#fragment'

        self.server_stderr = subprocess.PIPE
        self._run_http_fallback_test(options, 400)

    def test_version_check(self):
        """Tests http fallback on version check fail."""

        options = self._options
        options.version = 99
        self._run_http_fallback_test(options, 400)

    def test_basic_auth_connection(self):
        """Test successful basic auth"""

        options = self._options
        options.use_basic_auth = True

        self.server_stderr = subprocess.PIPE
        self._run_test_with_options(_check_handshake_with_basic_auth,
                                    options,
                                    server_options=['--basic-auth'])

    def test_invalid_basic_auth_connection(self):
        """Tests basic auth with invalid credentials"""

        options = self._options
        options.use_basic_auth = True
        options.basic_auth_credential = 'invalid:test'

        self.server_stderr = subprocess.PIPE

        with self.assertRaises(client_for_testing.HttpStatusException) as e:
            self._run_test_with_options(_check_handshake_with_basic_auth,
                                        options,
                                        server_options=['--basic-auth'])
            self.assertEqual(101, e.exception.status)


class EndToEndTestWithEchoClient(EndToEndTestBase):
    def setUp(self):
        EndToEndTestBase.setUp(self)

    def _check_example_echo_client_result(self, expected, stdoutdata,
                                          stderrdata):
        actual = stdoutdata.decode(locale.getpreferredencoding())

        # In Python 3 on Windows we get "\r\n" terminators back from
        # the subprocess and we need to replace them with "\n" to get
        # a match. This is a bit of a hack, but avoids platform- and
        # version- specific code.
        actual = actual.replace('\r\n', '\n')

        if actual != expected:
            raise Exception('Unexpected result on example echo client: '
                            '%r (expected) vs %r (actual)' %
                            (expected, actual))
        if stderrdata is not None:
            raise Exception('Unexpected error message on example echo '
                            'client: %r' % stderrdata)

    def test_example_echo_client(self):
        """Tests that the echo_client.py example can talk with the server."""

        server = self._run_server()
        try:
            time.sleep(_SERVER_WARMUP_IN_SEC)

            client_command = os.path.join(self.top_dir, 'example',
                                          'echo_client.py')

            # Expected output for the default messages.
            default_expectation = (u'Send: Hello\n'
                                   u'Recv: Hello\n'
                                   u'Send: <>\n'
                                   u'Recv: <>\n'
                                   u'Send close\n'
                                   u'Recv ack\n')

            args = [client_command, '-p', str(self._options.server_port)]
            client = self._run_python_command(args, stdout=subprocess.PIPE)
            stdoutdata, stderrdata = client.communicate()
            self._check_example_echo_client_result(default_expectation,
                                                   stdoutdata, stderrdata)

            # Process a big message for which extended payload length is used.
            # To handle extended payload length, ws_version attribute will be
            # accessed. This test checks that ws_version is correctly set.
            big_message = 'a' * 1024
            args = [
                client_command, '-p',
                str(self._options.server_port), '-m', big_message
            ]
            client = self._run_python_command(args, stdout=subprocess.PIPE)
            stdoutdata, stderrdata = client.communicate()
            expected = ('Send: %s\nRecv: %s\nSend close\nRecv ack\n' %
                        (big_message, big_message))
            self._check_example_echo_client_result(expected, stdoutdata,
                                                   stderrdata)

            # Test the permessage-deflate extension.
            args = [
                client_command, '-p',
                str(self._options.server_port), '--use_permessage_deflate'
            ]
            client = self._run_python_command(args, stdout=subprocess.PIPE)
            stdoutdata, stderrdata = client.communicate()
            self._check_example_echo_client_result(default_expectation,
                                                   stdoutdata, stderrdata)
        finally:
            self._close_server(server)


class EndToEndTestWithCgi(EndToEndTestBase):
    def setUp(self):
        EndToEndTestBase.setUp(self)

    def test_cgi(self):
        """Verifies that CGI scripts work."""

        server = self._run_server(extra_args=['--cgi-paths', '/cgi-bin'])
        time.sleep(_SERVER_WARMUP_IN_SEC)

        url = 'http://localhost:%d/cgi-bin/hi.py' % self._options.server_port

        # urlopen() in Python 2.7 doesn't support "with".
        try:
            f = urllib.request.urlopen(url)
        except:
            self._close_server(server)
            raise

        try:
            self.assertEqual(f.getcode(), 200)
            self.assertEqual(f.info().get('Content-Type'), 'text/plain')
            body = f.read()
            self.assertEqual(body.rstrip(b'\r\n'), b'Hi from hi.py')
        finally:
            f.close()
            self._close_server(server)


if __name__ == '__main__':
    unittest.main()

# vi:sts=4 sw=4 et
