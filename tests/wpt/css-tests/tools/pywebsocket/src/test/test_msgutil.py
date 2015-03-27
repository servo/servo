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


"""Tests for msgutil module."""


import array
import Queue
import random
import struct
import unittest
import zlib

import set_sys_path  # Update sys.path to locate mod_pywebsocket module.

from mod_pywebsocket import common
from mod_pywebsocket.extensions import DeflateFrameExtensionProcessor
from mod_pywebsocket.extensions import PerMessageCompressExtensionProcessor
from mod_pywebsocket.extensions import PerMessageDeflateExtensionProcessor
from mod_pywebsocket import msgutil
from mod_pywebsocket.stream import InvalidUTF8Exception
from mod_pywebsocket.stream import Stream
from mod_pywebsocket.stream import StreamHixie75
from mod_pywebsocket.stream import StreamOptions
from mod_pywebsocket import util
from test import mock


# We use one fixed nonce for testing instead of cryptographically secure PRNG.
_MASKING_NONCE = 'ABCD'


def _mask_hybi(frame):
    frame_key = map(ord, _MASKING_NONCE)
    frame_key_len = len(frame_key)
    result = array.array('B')
    result.fromstring(frame)
    count = 0
    for i in xrange(len(result)):
        result[i] ^= frame_key[count]
        count = (count + 1) % frame_key_len
    return _MASKING_NONCE + result.tostring()


def _install_extension_processor(processor, request, stream_options):
    response = processor.get_extension_response()
    if response is not None:
        processor.setup_stream_options(stream_options)
        request.ws_extension_processors.append(processor)


def _create_request_from_rawdata(
        read_data,
        deflate_frame_request=None,
        permessage_compression_request=None,
        permessage_deflate_request=None):
    req = mock.MockRequest(connection=mock.MockConn(''.join(read_data)))
    req.ws_version = common.VERSION_HYBI_LATEST
    req.ws_extension_processors = []

    processor = None
    if deflate_frame_request is not None:
        processor = DeflateFrameExtensionProcessor(deflate_frame_request)
    elif permessage_compression_request is not None:
        processor = PerMessageCompressExtensionProcessor(
                permessage_compression_request)
    elif permessage_deflate_request is not None:
        processor = PerMessageDeflateExtensionProcessor(
                permessage_deflate_request)

    stream_options = StreamOptions()
    if processor is not None:
        _install_extension_processor(processor, req, stream_options)
    req.ws_stream = Stream(req, stream_options)

    return req


def _create_request(*frames):
    """Creates MockRequest using data given as frames.

    frames will be returned on calling request.connection.read() where request
    is MockRequest returned by this function.
    """

    read_data = []
    for (header, body) in frames:
        read_data.append(header + _mask_hybi(body))

    return _create_request_from_rawdata(read_data)


def _create_blocking_request():
    """Creates MockRequest.

    Data written to a MockRequest can be read out by calling
    request.connection.written_data().
    """

    req = mock.MockRequest(connection=mock.MockBlockingConn())
    req.ws_version = common.VERSION_HYBI_LATEST
    stream_options = StreamOptions()
    req.ws_stream = Stream(req, stream_options)
    return req


def _create_request_hixie75(read_data=''):
    req = mock.MockRequest(connection=mock.MockConn(read_data))
    req.ws_stream = StreamHixie75(req)
    return req


def _create_blocking_request_hixie75():
    req = mock.MockRequest(connection=mock.MockBlockingConn())
    req.ws_stream = StreamHixie75(req)
    return req


class BasicMessageTest(unittest.TestCase):
    """Basic tests for Stream."""

    def test_send_message(self):
        request = _create_request()
        msgutil.send_message(request, 'Hello')
        self.assertEqual('\x81\x05Hello', request.connection.written_data())

        payload = 'a' * 125
        request = _create_request()
        msgutil.send_message(request, payload)
        self.assertEqual('\x81\x7d' + payload,
                         request.connection.written_data())

    def test_send_medium_message(self):
        payload = 'a' * 126
        request = _create_request()
        msgutil.send_message(request, payload)
        self.assertEqual('\x81\x7e\x00\x7e' + payload,
                         request.connection.written_data())

        payload = 'a' * ((1 << 16) - 1)
        request = _create_request()
        msgutil.send_message(request, payload)
        self.assertEqual('\x81\x7e\xff\xff' + payload,
                         request.connection.written_data())

    def test_send_large_message(self):
        payload = 'a' * (1 << 16)
        request = _create_request()
        msgutil.send_message(request, payload)
        self.assertEqual('\x81\x7f\x00\x00\x00\x00\x00\x01\x00\x00' + payload,
                         request.connection.written_data())

    def test_send_message_unicode(self):
        request = _create_request()
        msgutil.send_message(request, u'\u65e5')
        # U+65e5 is encoded as e6,97,a5 in UTF-8
        self.assertEqual('\x81\x03\xe6\x97\xa5',
                         request.connection.written_data())

    def test_send_message_fragments(self):
        request = _create_request()
        msgutil.send_message(request, 'Hello', False)
        msgutil.send_message(request, ' ', False)
        msgutil.send_message(request, 'World', False)
        msgutil.send_message(request, '!', True)
        self.assertEqual('\x01\x05Hello\x00\x01 \x00\x05World\x80\x01!',
                         request.connection.written_data())

    def test_send_fragments_immediate_zero_termination(self):
        request = _create_request()
        msgutil.send_message(request, 'Hello World!', False)
        msgutil.send_message(request, '', True)
        self.assertEqual('\x01\x0cHello World!\x80\x00',
                         request.connection.written_data())

    def test_receive_message(self):
        request = _create_request(
            ('\x81\x85', 'Hello'), ('\x81\x86', 'World!'))
        self.assertEqual('Hello', msgutil.receive_message(request))
        self.assertEqual('World!', msgutil.receive_message(request))

        payload = 'a' * 125
        request = _create_request(('\x81\xfd', payload))
        self.assertEqual(payload, msgutil.receive_message(request))

    def test_receive_medium_message(self):
        payload = 'a' * 126
        request = _create_request(('\x81\xfe\x00\x7e', payload))
        self.assertEqual(payload, msgutil.receive_message(request))

        payload = 'a' * ((1 << 16) - 1)
        request = _create_request(('\x81\xfe\xff\xff', payload))
        self.assertEqual(payload, msgutil.receive_message(request))

    def test_receive_large_message(self):
        payload = 'a' * (1 << 16)
        request = _create_request(
            ('\x81\xff\x00\x00\x00\x00\x00\x01\x00\x00', payload))
        self.assertEqual(payload, msgutil.receive_message(request))

    def test_receive_length_not_encoded_using_minimal_number_of_bytes(self):
        # Log warning on receiving bad payload length field that doesn't use
        # minimal number of bytes but continue processing.

        payload = 'a'
        # 1 byte can be represented without extended payload length field.
        request = _create_request(
            ('\x81\xff\x00\x00\x00\x00\x00\x00\x00\x01', payload))
        self.assertEqual(payload, msgutil.receive_message(request))

    def test_receive_message_unicode(self):
        request = _create_request(('\x81\x83', '\xe6\x9c\xac'))
        # U+672c is encoded as e6,9c,ac in UTF-8
        self.assertEqual(u'\u672c', msgutil.receive_message(request))

    def test_receive_message_erroneous_unicode(self):
        # \x80 and \x81 are invalid as UTF-8.
        request = _create_request(('\x81\x82', '\x80\x81'))
        # Invalid characters should raise InvalidUTF8Exception
        self.assertRaises(InvalidUTF8Exception,
                          msgutil.receive_message,
                          request)

    def test_receive_fragments(self):
        request = _create_request(
            ('\x01\x85', 'Hello'),
            ('\x00\x81', ' '),
            ('\x00\x85', 'World'),
            ('\x80\x81', '!'))
        self.assertEqual('Hello World!', msgutil.receive_message(request))

    def test_receive_fragments_unicode(self):
        # UTF-8 encodes U+6f22 into e6bca2 and U+5b57 into e5ad97.
        request = _create_request(
            ('\x01\x82', '\xe6\xbc'),
            ('\x00\x82', '\xa2\xe5'),
            ('\x80\x82', '\xad\x97'))
        self.assertEqual(u'\u6f22\u5b57', msgutil.receive_message(request))

    def test_receive_fragments_immediate_zero_termination(self):
        request = _create_request(
            ('\x01\x8c', 'Hello World!'), ('\x80\x80', ''))
        self.assertEqual('Hello World!', msgutil.receive_message(request))

    def test_receive_fragments_duplicate_start(self):
        request = _create_request(
            ('\x01\x85', 'Hello'), ('\x01\x85', 'World'))
        self.assertRaises(msgutil.InvalidFrameException,
                          msgutil.receive_message,
                          request)

    def test_receive_fragments_intermediate_but_not_started(self):
        request = _create_request(('\x00\x85', 'Hello'))
        self.assertRaises(msgutil.InvalidFrameException,
                          msgutil.receive_message,
                          request)

    def test_receive_fragments_end_but_not_started(self):
        request = _create_request(('\x80\x85', 'Hello'))
        self.assertRaises(msgutil.InvalidFrameException,
                          msgutil.receive_message,
                          request)

    def test_receive_message_discard(self):
        request = _create_request(
            ('\x8f\x86', 'IGNORE'), ('\x81\x85', 'Hello'),
            ('\x8f\x89', 'DISREGARD'), ('\x81\x86', 'World!'))
        self.assertRaises(msgutil.UnsupportedFrameException,
                          msgutil.receive_message, request)
        self.assertEqual('Hello', msgutil.receive_message(request))
        self.assertRaises(msgutil.UnsupportedFrameException,
                          msgutil.receive_message, request)
        self.assertEqual('World!', msgutil.receive_message(request))

    def test_receive_close(self):
        request = _create_request(
            ('\x88\x8a', struct.pack('!H', 1000) + 'Good bye'))
        self.assertEqual(None, msgutil.receive_message(request))
        self.assertEqual(1000, request.ws_close_code)
        self.assertEqual('Good bye', request.ws_close_reason)

    def test_send_longest_close(self):
        reason = 'a' * 123
        request = _create_request(
            ('\x88\xfd',
             struct.pack('!H', common.STATUS_NORMAL_CLOSURE) + reason))
        request.ws_stream.close_connection(common.STATUS_NORMAL_CLOSURE,
                                           reason)
        self.assertEqual(request.ws_close_code, common.STATUS_NORMAL_CLOSURE)
        self.assertEqual(request.ws_close_reason, reason)

    def test_send_close_too_long(self):
        request = _create_request()
        self.assertRaises(msgutil.BadOperationException,
                          Stream.close_connection,
                          request.ws_stream,
                          common.STATUS_NORMAL_CLOSURE,
                          'a' * 124)

    def test_send_close_inconsistent_code_and_reason(self):
        request = _create_request()
        # reason parameter must not be specified when code is None.
        self.assertRaises(msgutil.BadOperationException,
                          Stream.close_connection,
                          request.ws_stream,
                          None,
                          'a')

    def test_send_ping(self):
        request = _create_request()
        msgutil.send_ping(request, 'Hello World!')
        self.assertEqual('\x89\x0cHello World!',
                         request.connection.written_data())

    def test_send_longest_ping(self):
        request = _create_request()
        msgutil.send_ping(request, 'a' * 125)
        self.assertEqual('\x89\x7d' + 'a' * 125,
                         request.connection.written_data())

    def test_send_ping_too_long(self):
        request = _create_request()
        self.assertRaises(msgutil.BadOperationException,
                          msgutil.send_ping,
                          request,
                          'a' * 126)

    def test_receive_ping(self):
        """Tests receiving a ping control frame."""

        def handler(request, message):
            request.called = True

        # Stream automatically respond to ping with pong without any action
        # by application layer.
        request = _create_request(
            ('\x89\x85', 'Hello'), ('\x81\x85', 'World'))
        self.assertEqual('World', msgutil.receive_message(request))
        self.assertEqual('\x8a\x05Hello',
                         request.connection.written_data())

        request = _create_request(
            ('\x89\x85', 'Hello'), ('\x81\x85', 'World'))
        request.on_ping_handler = handler
        self.assertEqual('World', msgutil.receive_message(request))
        self.assertTrue(request.called)

    def test_receive_longest_ping(self):
        request = _create_request(
            ('\x89\xfd', 'a' * 125), ('\x81\x85', 'World'))
        self.assertEqual('World', msgutil.receive_message(request))
        self.assertEqual('\x8a\x7d' + 'a' * 125,
                         request.connection.written_data())

    def test_receive_ping_too_long(self):
        request = _create_request(('\x89\xfe\x00\x7e', 'a' * 126))
        self.assertRaises(msgutil.InvalidFrameException,
                          msgutil.receive_message,
                          request)

    def test_receive_pong(self):
        """Tests receiving a pong control frame."""

        def handler(request, message):
            request.called = True

        request = _create_request(
            ('\x8a\x85', 'Hello'), ('\x81\x85', 'World'))
        request.on_pong_handler = handler
        msgutil.send_ping(request, 'Hello')
        self.assertEqual('\x89\x05Hello',
                         request.connection.written_data())
        # Valid pong is received, but receive_message won't return for it.
        self.assertEqual('World', msgutil.receive_message(request))
        # Check that nothing was written after receive_message call.
        self.assertEqual('\x89\x05Hello',
                         request.connection.written_data())

        self.assertTrue(request.called)

    def test_receive_unsolicited_pong(self):
        # Unsolicited pong is allowed from HyBi 07.
        request = _create_request(
            ('\x8a\x85', 'Hello'), ('\x81\x85', 'World'))
        msgutil.receive_message(request)

        request = _create_request(
            ('\x8a\x85', 'Hello'), ('\x81\x85', 'World'))
        msgutil.send_ping(request, 'Jumbo')
        # Body mismatch.
        msgutil.receive_message(request)

    def test_ping_cannot_be_fragmented(self):
        request = _create_request(('\x09\x85', 'Hello'))
        self.assertRaises(msgutil.InvalidFrameException,
                          msgutil.receive_message,
                          request)

    def test_ping_with_too_long_payload(self):
        request = _create_request(('\x89\xfe\x01\x00', 'a' * 256))
        self.assertRaises(msgutil.InvalidFrameException,
                          msgutil.receive_message,
                          request)


class DeflateFrameTest(unittest.TestCase):
    """Tests for checking deflate-frame extension."""

    def test_send_message(self):
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        request = _create_request_from_rawdata(
            '', deflate_frame_request=extension)
        msgutil.send_message(request, 'Hello')
        msgutil.send_message(request, 'World')

        expected = ''

        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        expected += '\xc1%c' % len(compressed_hello)
        expected += compressed_hello

        compressed_world = compress.compress('World')
        compressed_world += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_world = compressed_world[:-4]
        expected += '\xc1%c' % len(compressed_world)
        expected += compressed_world

        self.assertEqual(expected, request.connection.written_data())

    def test_send_message_bfinal(self):
        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        request = _create_request_from_rawdata(
            '', deflate_frame_request=extension)
        self.assertEquals(1, len(request.ws_extension_processors))
        deflate_frame_processor = request.ws_extension_processors[0]
        deflate_frame_processor.set_bfinal(True)
        msgutil.send_message(request, 'Hello')
        msgutil.send_message(request, 'World')

        expected = ''

        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_FINISH)
        compressed_hello = compressed_hello + chr(0)
        expected += '\xc1%c' % len(compressed_hello)
        expected += compressed_hello

        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_world = compress.compress('World')
        compressed_world += compress.flush(zlib.Z_FINISH)
        compressed_world = compressed_world + chr(0)
        expected += '\xc1%c' % len(compressed_world)
        expected += compressed_world

        self.assertEqual(expected, request.connection.written_data())

    def test_send_message_comp_bit(self):
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        request = _create_request_from_rawdata(
            '', deflate_frame_request=extension)
        self.assertEquals(1, len(request.ws_extension_processors))
        deflate_frame_processor = request.ws_extension_processors[0]
        msgutil.send_message(request, 'Hello')
        deflate_frame_processor.disable_outgoing_compression()
        msgutil.send_message(request, 'Hello')
        deflate_frame_processor.enable_outgoing_compression()
        msgutil.send_message(request, 'Hello')

        expected = ''

        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        expected += '\xc1%c' % len(compressed_hello)
        expected += compressed_hello

        expected += '\x81\x05Hello'

        compressed_2nd_hello = compress.compress('Hello')
        compressed_2nd_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_2nd_hello = compressed_2nd_hello[:-4]
        expected += '\xc1%c' % len(compressed_2nd_hello)
        expected += compressed_2nd_hello

        self.assertEqual(expected, request.connection.written_data())

    def test_send_message_no_context_takeover_parameter(self):
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        extension.add_parameter('no_context_takeover', None)
        request = _create_request_from_rawdata(
            '', deflate_frame_request=extension)
        for i in xrange(3):
            msgutil.send_message(request, 'Hello')

        compressed_message = compress.compress('Hello')
        compressed_message += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_message = compressed_message[:-4]
        expected = '\xc1%c' % len(compressed_message)
        expected += compressed_message

        self.assertEqual(
            expected + expected + expected, request.connection.written_data())

    def test_bad_request_parameters(self):
        """Tests that if there's anything wrong with deflate-frame extension
        request, deflate-frame is rejected.
        """

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        # max_window_bits less than 8 is illegal.
        extension.add_parameter('max_window_bits', '7')
        processor = DeflateFrameExtensionProcessor(extension)
        self.assertEqual(None, processor.get_extension_response())

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        # max_window_bits greater than 15 is illegal.
        extension.add_parameter('max_window_bits', '16')
        processor = DeflateFrameExtensionProcessor(extension)
        self.assertEqual(None, processor.get_extension_response())

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        # Non integer max_window_bits is illegal.
        extension.add_parameter('max_window_bits', 'foobar')
        processor = DeflateFrameExtensionProcessor(extension)
        self.assertEqual(None, processor.get_extension_response())

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        # no_context_takeover must not have any value.
        extension.add_parameter('no_context_takeover', 'foobar')
        processor = DeflateFrameExtensionProcessor(extension)
        self.assertEqual(None, processor.get_extension_response())

    def test_response_parameters(self):
        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        processor = DeflateFrameExtensionProcessor(extension)
        processor.set_response_window_bits(8)
        response = processor.get_extension_response()
        self.assertTrue(response.has_parameter('max_window_bits'))
        self.assertEqual('8', response.get_parameter_value('max_window_bits'))

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        processor = DeflateFrameExtensionProcessor(extension)
        processor.set_response_no_context_takeover(True)
        response = processor.get_extension_response()
        self.assertTrue(response.has_parameter('no_context_takeover'))
        self.assertTrue(
            response.get_parameter_value('no_context_takeover') is None)

    def test_receive_message(self):
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        data = ''

        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        data += '\xc1%c' % (len(compressed_hello) | 0x80)
        data += _mask_hybi(compressed_hello)

        compressed_websocket = compress.compress('WebSocket')
        compressed_websocket += compress.flush(zlib.Z_FINISH)
        compressed_websocket += '\x00'
        data += '\xc1%c' % (len(compressed_websocket) | 0x80)
        data += _mask_hybi(compressed_websocket)

        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        compressed_world = compress.compress('World')
        compressed_world += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_world = compressed_world[:-4]
        data += '\xc1%c' % (len(compressed_world) | 0x80)
        data += _mask_hybi(compressed_world)

        # Close frame
        data += '\x88\x8a' + _mask_hybi(struct.pack('!H', 1000) + 'Good bye')

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        request = _create_request_from_rawdata(
            data, deflate_frame_request=extension)
        self.assertEqual('Hello', msgutil.receive_message(request))
        self.assertEqual('WebSocket', msgutil.receive_message(request))
        self.assertEqual('World', msgutil.receive_message(request))

        self.assertEqual(None, msgutil.receive_message(request))

    def test_receive_message_client_using_smaller_window(self):
        """Test that frames coming from a client which is using smaller window
        size that the server are correctly received.
        """

        # Using the smallest window bits of 8 for generating input frames.
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -8)

        data = ''

        # Use a frame whose content is bigger than the clients' DEFLATE window
        # size before compression. The content mainly consists of 'a' but
        # repetition of 'b' is put at the head and tail so that if the window
        # size is big, the head is back-referenced but if small, not.
        payload = 'b' * 64 + 'a' * 1024 + 'b' * 64
        compressed_hello = compress.compress(payload)
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        data += '\xc1%c' % (len(compressed_hello) | 0x80)
        data += _mask_hybi(compressed_hello)

        # Close frame
        data += '\x88\x8a' + _mask_hybi(struct.pack('!H', 1000) + 'Good bye')

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        request = _create_request_from_rawdata(
            data, deflate_frame_request=extension)
        self.assertEqual(payload, msgutil.receive_message(request))

        self.assertEqual(None, msgutil.receive_message(request))

    def test_receive_message_comp_bit(self):
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        data = ''

        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        data += '\xc1%c' % (len(compressed_hello) | 0x80)
        data += _mask_hybi(compressed_hello)

        data += '\x81\x85' + _mask_hybi('Hello')

        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        compressed_2nd_hello = compress.compress('Hello')
        compressed_2nd_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_2nd_hello = compressed_2nd_hello[:-4]
        data += '\xc1%c' % (len(compressed_2nd_hello) | 0x80)
        data += _mask_hybi(compressed_2nd_hello)

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        request = _create_request_from_rawdata(
            data, deflate_frame_request=extension)
        for i in xrange(3):
            self.assertEqual('Hello', msgutil.receive_message(request))

    def test_receive_message_various_btype(self):
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        data = ''

        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        data += '\xc1%c' % (len(compressed_hello) | 0x80)
        data += _mask_hybi(compressed_hello)

        compressed_websocket = compress.compress('WebSocket')
        compressed_websocket += compress.flush(zlib.Z_FINISH)
        compressed_websocket += '\x00'
        data += '\xc1%c' % (len(compressed_websocket) | 0x80)
        data += _mask_hybi(compressed_websocket)

        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        compressed_world = compress.compress('World')
        compressed_world += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_world = compressed_world[:-4]
        data += '\xc1%c' % (len(compressed_world) | 0x80)
        data += _mask_hybi(compressed_world)

        # Close frame
        data += '\x88\x8a' + _mask_hybi(struct.pack('!H', 1000) + 'Good bye')

        extension = common.ExtensionParameter(common.DEFLATE_FRAME_EXTENSION)
        request = _create_request_from_rawdata(
            data, deflate_frame_request=extension)
        self.assertEqual('Hello', msgutil.receive_message(request))
        self.assertEqual('WebSocket', msgutil.receive_message(request))
        self.assertEqual('World', msgutil.receive_message(request))

        self.assertEqual(None, msgutil.receive_message(request))


class PerMessageDeflateTest(unittest.TestCase):
    """Tests for permessage-deflate extension."""

    def test_send_message(self):
        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)
        msgutil.send_message(request, 'Hello')

        compress = zlib.compressobj(
                zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        expected = '\xc1%c' % len(compressed_hello)
        expected += compressed_hello
        self.assertEqual(expected, request.connection.written_data())

    def test_send_empty_message(self):
        """Test that an empty message is compressed correctly."""

        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)

        msgutil.send_message(request, '')

        # Payload in binary: 0b00000010 0b00000000
        # From LSB,
        # - 1 bit of BFINAL (0)
        # - 2 bits of BTYPE (01 that means fixed Huffman)
        # - 7 bits of the first code (0000000 that is the code for the
        #   end-of-block)
        # - 1 bit of BFINAL (0)
        # - 2 bits of BTYPE (no compression)
        # - 3 bits of padding
        self.assertEqual('\xc1\x02\x02\x00',
                         request.connection.written_data())

    def test_send_message_with_null_character(self):
        """Test that a simple payload (one null) is framed correctly."""

        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)

        msgutil.send_message(request, '\x00')

        # Payload in binary: 0b01100010 0b00000000 0b00000000
        # From LSB,
        # - 1 bit of BFINAL (0)
        # - 2 bits of BTYPE (01 that means fixed Huffman)
        # - 8 bits of the first code (00110000 that is the code for the literal
        #   alphabet 0x00)
        # - 7 bits of the second code (0000000 that is the code for the
        #   end-of-block)
        # - 1 bit of BFINAL (0)
        # - 2 bits of BTYPE (no compression)
        # - 2 bits of padding
        self.assertEqual('\xc1\x03\x62\x00\x00',
                         request.connection.written_data())

    def test_send_two_messages(self):
        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)
        msgutil.send_message(request, 'Hello')
        msgutil.send_message(request, 'World')

        compress = zlib.compressobj(
                zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        expected = ''

        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        expected += '\xc1%c' % len(compressed_hello)
        expected += compressed_hello

        compressed_world = compress.compress('World')
        compressed_world += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_world = compressed_world[:-4]
        expected += '\xc1%c' % len(compressed_world)
        expected += compressed_world

        self.assertEqual(expected, request.connection.written_data())

    def test_send_message_fragmented(self):
        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)
        msgutil.send_message(request, 'Hello', end=False)
        msgutil.send_message(request, 'Goodbye', end=False)
        msgutil.send_message(request, 'World')

        compress = zlib.compressobj(
                zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        expected = '\x41%c' % len(compressed_hello)
        expected += compressed_hello
        compressed_goodbye = compress.compress('Goodbye')
        compressed_goodbye += compress.flush(zlib.Z_SYNC_FLUSH)
        expected += '\x00%c' % len(compressed_goodbye)
        expected += compressed_goodbye
        compressed_world = compress.compress('World')
        compressed_world += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_world = compressed_world[:-4]
        expected += '\x80%c' % len(compressed_world)
        expected += compressed_world
        self.assertEqual(expected, request.connection.written_data())

    def test_send_message_fragmented_empty_first_frame(self):
        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)
        msgutil.send_message(request, '', end=False)
        msgutil.send_message(request, 'Hello')

        compress = zlib.compressobj(
                zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello = compress.compress('')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        expected = '\x41%c' % len(compressed_hello)
        expected += compressed_hello
        compressed_empty = compress.compress('Hello')
        compressed_empty += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_empty = compressed_empty[:-4]
        expected += '\x80%c' % len(compressed_empty)
        expected += compressed_empty
        print '%r' % expected
        self.assertEqual(expected, request.connection.written_data())

    def test_send_message_fragmented_empty_last_frame(self):
        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)
        msgutil.send_message(request, 'Hello', end=False)
        msgutil.send_message(request, '')

        compress = zlib.compressobj(
                zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        expected = '\x41%c' % len(compressed_hello)
        expected += compressed_hello
        compressed_empty = compress.compress('')
        compressed_empty += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_empty = compressed_empty[:-4]
        expected += '\x80%c' % len(compressed_empty)
        expected += compressed_empty
        self.assertEqual(expected, request.connection.written_data())

    def test_send_message_using_small_window(self):
        common_part = 'abcdefghijklmnopqrstuvwxyz'
        test_message = common_part + '-' * 30000 + common_part

        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        extension.add_parameter('server_max_window_bits', '8')
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)
        msgutil.send_message(request, test_message)

        expected_websocket_header_size = 2
        expected_websocket_payload_size = 91

        actual_frame = request.connection.written_data()
        self.assertEqual(expected_websocket_header_size +
                         expected_websocket_payload_size,
                         len(actual_frame))
        actual_header = actual_frame[0:expected_websocket_header_size]
        actual_payload = actual_frame[expected_websocket_header_size:]

        self.assertEqual(
                '\xc1%c' % expected_websocket_payload_size, actual_header)
        decompress = zlib.decompressobj(-8)
        decompressed_message = decompress.decompress(
                actual_payload + '\x00\x00\xff\xff')
        decompressed_message += decompress.flush()
        self.assertEqual(test_message, decompressed_message)
        self.assertEqual(0, len(decompress.unused_data))
        self.assertEqual(0, len(decompress.unconsumed_tail))

    def test_send_message_no_context_takeover_parameter(self):
        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        extension.add_parameter('server_no_context_takeover', None)
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)
        for i in xrange(3):
            msgutil.send_message(request, 'Hello', end=False)
            msgutil.send_message(request, 'Hello', end=True)

        compress = zlib.compressobj(
                zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        first_hello = compress.compress('Hello')
        first_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        expected = '\x41%c' % len(first_hello)
        expected += first_hello
        second_hello = compress.compress('Hello')
        second_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        second_hello = second_hello[:-4]
        expected += '\x80%c' % len(second_hello)
        expected += second_hello

        self.assertEqual(
                expected + expected + expected,
                request.connection.written_data())

    def test_send_message_fragmented_bfinal(self):
        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                '', permessage_deflate_request=extension)
        self.assertEquals(1, len(request.ws_extension_processors))
        request.ws_extension_processors[0].set_bfinal(True)
        msgutil.send_message(request, 'Hello', end=False)
        msgutil.send_message(request, 'World', end=True)

        expected = ''

        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_FINISH)
        compressed_hello = compressed_hello + chr(0)
        expected += '\x41%c' % len(compressed_hello)
        expected += compressed_hello

        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_world = compress.compress('World')
        compressed_world += compress.flush(zlib.Z_FINISH)
        compressed_world = compressed_world + chr(0)
        expected += '\x80%c' % len(compressed_world)
        expected += compressed_world

        self.assertEqual(expected, request.connection.written_data())

    def test_receive_message_deflate(self):
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        data = '\xc1%c' % (len(compressed_hello) | 0x80)
        data += _mask_hybi(compressed_hello)

        # Close frame
        data += '\x88\x8a' + _mask_hybi(struct.pack('!H', 1000) + 'Good bye')

        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                data, permessage_deflate_request=extension)
        self.assertEqual('Hello', msgutil.receive_message(request))

        self.assertEqual(None, msgutil.receive_message(request))

    def test_receive_message_random_section(self):
        """Test that a compressed message fragmented into lots of chunks is
        correctly received.
        """

        random.seed(a=0)
        payload = ''.join(
            [chr(random.randint(0, 255)) for i in xrange(1000)])

        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_payload = compress.compress(payload)
        compressed_payload += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_payload = compressed_payload[:-4]

        # Fragment the compressed payload into lots of frames.
        bytes_chunked = 0
        data = ''
        frame_count = 0

        chunk_sizes = []

        while bytes_chunked < len(compressed_payload):
            # Make sure that
            # - the length of chunks are equal or less than 125 so that we can
            #   use 1 octet length header format for all frames.
            # - at least 10 chunks are created.
            chunk_size = random.randint(
                1, min(125,
                       len(compressed_payload) / 10,
                       len(compressed_payload) - bytes_chunked))
            chunk_sizes.append(chunk_size)
            chunk = compressed_payload[
                bytes_chunked:bytes_chunked + chunk_size]
            bytes_chunked += chunk_size

            first_octet = 0x00
            if len(data) == 0:
                first_octet = first_octet | 0x42
            if bytes_chunked == len(compressed_payload):
                first_octet = first_octet | 0x80

            data += '%c%c' % (first_octet, chunk_size | 0x80)
            data += _mask_hybi(chunk)

            frame_count += 1

        print "Chunk sizes: %r" % chunk_sizes
        self.assertTrue(len(chunk_sizes) > 10)

        # Close frame
        data += '\x88\x8a' + _mask_hybi(struct.pack('!H', 1000) + 'Good bye')

        extension = common.ExtensionParameter(
            common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
            data, permessage_deflate_request=extension)
        self.assertEqual(payload, msgutil.receive_message(request))

        self.assertEqual(None, msgutil.receive_message(request))

    def test_receive_two_messages(self):
        compress = zlib.compressobj(
                zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        data = ''

        compressed_hello = compress.compress('HelloWebSocket')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        split_position = len(compressed_hello) / 2
        data += '\x41%c' % (split_position | 0x80)
        data += _mask_hybi(compressed_hello[:split_position])

        data += '\x80%c' % ((len(compressed_hello) - split_position) | 0x80)
        data += _mask_hybi(compressed_hello[split_position:])

        compress = zlib.compressobj(
                zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)

        compressed_world = compress.compress('World')
        compressed_world += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_world = compressed_world[:-4]
        data += '\xc1%c' % (len(compressed_world) | 0x80)
        data += _mask_hybi(compressed_world)

        # Close frame
        data += '\x88\x8a' + _mask_hybi(struct.pack('!H', 1000) + 'Good bye')

        extension = common.ExtensionParameter(
                common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
                data, permessage_deflate_request=extension)
        self.assertEqual('HelloWebSocket', msgutil.receive_message(request))
        self.assertEqual('World', msgutil.receive_message(request))

        self.assertEqual(None, msgutil.receive_message(request))

    def test_receive_message_mixed_btype(self):
        """Test that a message compressed using lots of DEFLATE blocks with
        various flush mode is correctly received.
        """

        random.seed(a=0)
        payload = ''.join(
            [chr(random.randint(0, 255)) for i in xrange(1000)])

        compress = None

        # Fragment the compressed payload into lots of frames.
        bytes_chunked = 0
        compressed_payload = ''

        chunk_sizes = []
        methods = []
        sync_used = False
        finish_used = False

        while bytes_chunked < len(payload):
            # Make sure at least 10 chunks are created.
            chunk_size = random.randint(
                1, min(100, len(payload) - bytes_chunked))
            chunk_sizes.append(chunk_size)
            chunk = payload[bytes_chunked:bytes_chunked + chunk_size]

            bytes_chunked += chunk_size

            if compress is None:
                compress = zlib.compressobj(
                    zlib.Z_DEFAULT_COMPRESSION,
                    zlib.DEFLATED,
                    -zlib.MAX_WBITS)

            if bytes_chunked == len(payload):
                compressed_payload += compress.compress(chunk)
                compressed_payload += compress.flush(zlib.Z_SYNC_FLUSH)
                compressed_payload = compressed_payload[:-4]
            else:
                method = random.randint(0, 1)
                methods.append(method)
                if method == 0:
                    compressed_payload += compress.compress(chunk)
                    compressed_payload += compress.flush(zlib.Z_SYNC_FLUSH)
                    sync_used = True
                else:
                    compressed_payload += compress.compress(chunk)
                    compressed_payload += compress.flush(zlib.Z_FINISH)
                    compress = None
                    finish_used = True

        print "Chunk sizes: %r" % chunk_sizes
        self.assertTrue(len(chunk_sizes) > 10)
        print "Methods: %r" % methods
        self.assertTrue(sync_used)
        self.assertTrue(finish_used)

        self.assertTrue(125 < len(compressed_payload))
        self.assertTrue(len(compressed_payload) < 65536)
        data = '\xc2\xfe' + struct.pack('!H', len(compressed_payload))
        data += _mask_hybi(compressed_payload)

        # Close frame
        data += '\x88\x8a' + _mask_hybi(struct.pack('!H', 1000) + 'Good bye')

        extension = common.ExtensionParameter(
            common.PERMESSAGE_DEFLATE_EXTENSION)
        request = _create_request_from_rawdata(
            data, permessage_deflate_request=extension)
        self.assertEqual(payload, msgutil.receive_message(request))

        self.assertEqual(None, msgutil.receive_message(request))


class PerMessageCompressTest(unittest.TestCase):
    """Tests for checking permessage-compression extension."""

    def test_deflate_response_parameters(self):
        extension = common.ExtensionParameter(
            common.PERMESSAGE_COMPRESSION_EXTENSION)
        extension.add_parameter('method', 'deflate')
        processor = PerMessageCompressExtensionProcessor(extension)
        response = processor.get_extension_response()
        self.assertEqual('deflate',
                         response.get_parameter_value('method'))

        extension = common.ExtensionParameter(
            common.PERMESSAGE_COMPRESSION_EXTENSION)
        extension.add_parameter('method', 'deflate')
        processor = PerMessageCompressExtensionProcessor(extension)

        def _compression_processor_hook(compression_processor):
            compression_processor.set_client_max_window_bits(8)
            compression_processor.set_client_no_context_takeover(True)
        processor.set_compression_processor_hook(
            _compression_processor_hook)
        response = processor.get_extension_response()
        self.assertEqual(
            'deflate; client_max_window_bits=8; client_no_context_takeover',
            response.get_parameter_value('method'))


class MessageTestHixie75(unittest.TestCase):
    """Tests for draft-hixie-thewebsocketprotocol-76 stream class."""

    def test_send_message(self):
        request = _create_request_hixie75()
        msgutil.send_message(request, 'Hello')
        self.assertEqual('\x00Hello\xff', request.connection.written_data())

    def test_send_message_unicode(self):
        request = _create_request_hixie75()
        msgutil.send_message(request, u'\u65e5')
        # U+65e5 is encoded as e6,97,a5 in UTF-8
        self.assertEqual('\x00\xe6\x97\xa5\xff',
                         request.connection.written_data())

    def test_receive_message(self):
        request = _create_request_hixie75('\x00Hello\xff\x00World!\xff')
        self.assertEqual('Hello', msgutil.receive_message(request))
        self.assertEqual('World!', msgutil.receive_message(request))

    def test_receive_message_unicode(self):
        request = _create_request_hixie75('\x00\xe6\x9c\xac\xff')
        # U+672c is encoded as e6,9c,ac in UTF-8
        self.assertEqual(u'\u672c', msgutil.receive_message(request))

    def test_receive_message_erroneous_unicode(self):
        # \x80 and \x81 are invalid as UTF-8.
        request = _create_request_hixie75('\x00\x80\x81\xff')
        # Invalid characters should be replaced with
        # U+fffd REPLACEMENT CHARACTER
        self.assertEqual(u'\ufffd\ufffd', msgutil.receive_message(request))

    def test_receive_message_discard(self):
        request = _create_request_hixie75('\x80\x06IGNORE\x00Hello\xff'
                                          '\x01DISREGARD\xff\x00World!\xff')
        self.assertEqual('Hello', msgutil.receive_message(request))
        self.assertEqual('World!', msgutil.receive_message(request))


class MessageReceiverTest(unittest.TestCase):
    """Tests the Stream class using MessageReceiver."""

    def test_queue(self):
        request = _create_blocking_request()
        receiver = msgutil.MessageReceiver(request)

        self.assertEqual(None, receiver.receive_nowait())

        request.connection.put_bytes('\x81\x86' + _mask_hybi('Hello!'))
        self.assertEqual('Hello!', receiver.receive())

    def test_onmessage(self):
        onmessage_queue = Queue.Queue()

        def onmessage_handler(message):
            onmessage_queue.put(message)

        request = _create_blocking_request()
        receiver = msgutil.MessageReceiver(request, onmessage_handler)

        request.connection.put_bytes('\x81\x86' + _mask_hybi('Hello!'))
        self.assertEqual('Hello!', onmessage_queue.get())


class MessageReceiverHixie75Test(unittest.TestCase):
    """Tests the StreamHixie75 class using MessageReceiver."""

    def test_queue(self):
        request = _create_blocking_request_hixie75()
        receiver = msgutil.MessageReceiver(request)

        self.assertEqual(None, receiver.receive_nowait())

        request.connection.put_bytes('\x00Hello!\xff')
        self.assertEqual('Hello!', receiver.receive())

    def test_onmessage(self):
        onmessage_queue = Queue.Queue()

        def onmessage_handler(message):
            onmessage_queue.put(message)

        request = _create_blocking_request_hixie75()
        receiver = msgutil.MessageReceiver(request, onmessage_handler)

        request.connection.put_bytes('\x00Hello!\xff')
        self.assertEqual('Hello!', onmessage_queue.get())


class MessageSenderTest(unittest.TestCase):
    """Tests the Stream class using MessageSender."""

    def test_send(self):
        request = _create_blocking_request()
        sender = msgutil.MessageSender(request)

        sender.send('World')
        self.assertEqual('\x81\x05World', request.connection.written_data())

    def test_send_nowait(self):
        # Use a queue to check the bytes written by MessageSender.
        # request.connection.written_data() cannot be used here because
        # MessageSender runs in a separate thread.
        send_queue = Queue.Queue()

        def write(bytes):
            send_queue.put(bytes)

        request = _create_blocking_request()
        request.connection.write = write

        sender = msgutil.MessageSender(request)

        sender.send_nowait('Hello')
        sender.send_nowait('World')
        self.assertEqual('\x81\x05Hello', send_queue.get())
        self.assertEqual('\x81\x05World', send_queue.get())


class MessageSenderHixie75Test(unittest.TestCase):
    """Tests the StreamHixie75 class using MessageSender."""

    def test_send(self):
        request = _create_blocking_request_hixie75()
        sender = msgutil.MessageSender(request)

        sender.send('World')
        self.assertEqual('\x00World\xff', request.connection.written_data())

    def test_send_nowait(self):
        # Use a queue to check the bytes written by MessageSender.
        # request.connection.written_data() cannot be used here because
        # MessageSender runs in a separate thread.
        send_queue = Queue.Queue()

        def write(bytes):
            send_queue.put(bytes)

        request = _create_blocking_request_hixie75()
        request.connection.write = write

        sender = msgutil.MessageSender(request)

        sender.send_nowait('Hello')
        sender.send_nowait('World')
        self.assertEqual('\x00Hello\xff', send_queue.get())
        self.assertEqual('\x00World\xff', send_queue.get())


if __name__ == '__main__':
    unittest.main()


# vi:sts=4 sw=4 et
