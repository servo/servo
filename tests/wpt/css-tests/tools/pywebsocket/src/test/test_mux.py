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


"""Tests for mux module."""

import Queue
import copy
import logging
import optparse
import struct
import sys
import unittest
import time
import zlib

import set_sys_path  # Update sys.path to locate mod_pywebsocket module.

from mod_pywebsocket import common
from mod_pywebsocket import mux
from mod_pywebsocket._stream_base import ConnectionTerminatedException
from mod_pywebsocket._stream_base import UnsupportedFrameException
from mod_pywebsocket._stream_hybi import Frame
from mod_pywebsocket._stream_hybi import Stream
from mod_pywebsocket._stream_hybi import StreamOptions
from mod_pywebsocket._stream_hybi import create_binary_frame
from mod_pywebsocket._stream_hybi import create_close_frame
from mod_pywebsocket._stream_hybi import create_closing_handshake_body
from mod_pywebsocket._stream_hybi import parse_frame
from mod_pywebsocket.extensions import MuxExtensionProcessor


import mock


_TEST_HEADERS = {'Host': 'server.example.com',
                 'Upgrade': 'websocket',
                 'Connection': 'Upgrade',
                 'Sec-WebSocket-Key': 'dGhlIHNhbXBsZSBub25jZQ==',
                 'Sec-WebSocket-Version': '13',
                 'Origin': 'http://example.com'}


class _OutgoingChannelData(object):
    def __init__(self):
        self.messages = []
        self.control_messages = []

        self.builder = mux._InnerMessageBuilder()

class _MockMuxConnection(mock.MockBlockingConn):
    """Mock class of mod_python connection for mux."""

    def __init__(self):
        mock.MockBlockingConn.__init__(self)
        self._control_blocks = []
        self._channel_data = {}

        self._current_opcode = None
        self._pending_fragments = []

        self.server_close_code = None

    def write(self, data):
        """Override MockBlockingConn.write."""

        self._current_data = data
        self._position = 0

        def _receive_bytes(length):
            if self._position + length > len(self._current_data):
                raise ConnectionTerminatedException(
                    'Failed to receive %d bytes from encapsulated '
                    'frame' % length)
            data = self._current_data[self._position:self._position+length]
            self._position += length
            return data

        # Parse physical frames and assemble a message if the message is
        # fragmented.
        opcode, payload, fin, rsv1, rsv2, rsv3 = (
            parse_frame(_receive_bytes, unmask_receive=False))

        self._pending_fragments.append(payload)

        if self._current_opcode is None:
            if opcode == common.OPCODE_CONTINUATION:
                raise Exception('Sending invalid continuation opcode')
            self._current_opcode = opcode
        else:
            if opcode != common.OPCODE_CONTINUATION:
                raise Exception('Sending invalid opcode %d' % opcode)
        if not fin:
            return

        inner_frame_data = ''.join(self._pending_fragments)
        self._pending_fragments = []
        self._current_opcode = None

        # Handle a control message on the physical channel.
        # TODO(bashi): Support other opcodes if needed.
        if opcode == common.OPCODE_CLOSE:
            if len(payload) >= 2:
                self.server_close_code = struct.unpack('!H', payload[:2])[0]
            close_body = create_closing_handshake_body(
                common.STATUS_NORMAL_CLOSURE, '')
            close_frame = create_close_frame(close_body, mask=True)
            self.put_bytes(close_frame)
            return

        # Parse the payload of the message on physical channel.
        parser = mux._MuxFramePayloadParser(inner_frame_data)
        channel_id = parser.read_channel_id()
        if channel_id == mux._CONTROL_CHANNEL_ID:
            self._control_blocks.extend(list(parser.read_control_blocks()))
            return

        if not channel_id in self._channel_data:
            self._channel_data[channel_id] = _OutgoingChannelData()
        channel_data = self._channel_data[channel_id]

        # Parse logical frames and assemble an inner (logical) message.
        (inner_fin, inner_rsv1, inner_rsv2, inner_rsv3, inner_opcode,
         inner_payload) = parser.read_inner_frame()
        inner_frame = Frame(inner_fin, inner_rsv1, inner_rsv2, inner_rsv3,
                            inner_opcode, inner_payload)
        message = channel_data.builder.build(inner_frame)
        if message is None:
            return

        if (message.opcode == common.OPCODE_TEXT or
            message.opcode == common.OPCODE_BINARY):
            channel_data.messages.append(message.payload)

            self.on_data_message(message.payload)
        else:
            channel_data.control_messages.append(
                {'opcode': message.opcode,
                 'message': message.payload})

    def on_data_message(self, message):
        pass

    def get_written_control_blocks(self):
        return self._control_blocks

    def get_written_messages(self, channel_id):
        return self._channel_data[channel_id].messages

    def get_written_control_messages(self, channel_id):
        return self._channel_data[channel_id].control_messages


class _FailOnWriteConnection(_MockMuxConnection):
    """Specicialized version of _MockMuxConnection. Its write() method raises
    an exception for testing when a data message is written.
    """

    def on_data_message(self, message):
        """Override to raise an exception."""

        raise Exception('Intentional failure')


class _ChannelEvent(object):
    """A structure that records channel events."""

    def __init__(self):
        self.request = None
        self.messages = []
        self.exception = None
        self.client_initiated_closing = False


class _MuxMockDispatcher(object):
    """Mock class of dispatch.Dispatcher for mux."""

    def __init__(self):
        self.channel_events = {}

    def do_extra_handshake(self, request):
        if request.ws_requested_protocols is not None:
            request.ws_protocol = request.ws_requested_protocols[0]

    def _do_echo(self, request, channel_events):
        while True:
            message = request.ws_stream.receive_message()
            if message == None:
                channel_events.client_initiated_closing = True
                return
            if message == 'Goodbye':
                return
            channel_events.messages.append(message)
            # echo back
            request.ws_stream.send_message(message)

    def _do_ping(self, request, channel_events):
        request.ws_stream.send_ping('Ping!')

    def _do_ping_while_hello_world(self, request, channel_events):
        request.ws_stream.send_message('Hello ', end=False)
        request.ws_stream.send_ping('Ping!')
        request.ws_stream.send_message('World!', end=True)

    def _do_two_ping_while_hello_world(self, request, channel_events):
        request.ws_stream.send_message('Hello ', end=False)
        request.ws_stream.send_ping('Ping!')
        request.ws_stream.send_ping('Pong!')
        request.ws_stream.send_message('World!', end=True)

    def transfer_data(self, request):
        self.channel_events[request.channel_id] = _ChannelEvent()
        self.channel_events[request.channel_id].request = request

        try:
            # Note: more handler will be added.
            if request.uri.endswith('echo'):
                self._do_echo(request,
                              self.channel_events[request.channel_id])
            elif request.uri.endswith('ping'):
                self._do_ping(request,
                              self.channel_events[request.channel_id])
            elif request.uri.endswith('two_ping_while_hello_world'):
                self._do_two_ping_while_hello_world(
                    request, self.channel_events[request.channel_id])
            elif request.uri.endswith('ping_while_hello_world'):
                self._do_ping_while_hello_world(
                    request, self.channel_events[request.channel_id])
            else:
                raise ValueError('Cannot handle path %r' % request.path)
            if not request.server_terminated:
                request.ws_stream.close_connection()
        except ConnectionTerminatedException, e:
            self.channel_events[request.channel_id].exception = e
        except Exception, e:
            self.channel_events[request.channel_id].exception = e
            raise


def _create_mock_request(connection=None, logical_channel_extensions=None):
    if connection is None:
        connection = _MockMuxConnection()

    request = mock.MockRequest(uri='/echo',
                               headers_in=_TEST_HEADERS,
                               connection=connection)
    request.ws_stream = Stream(request, options=StreamOptions())
    request.mux_processor = MuxExtensionProcessor(
        common.ExtensionParameter(common.MUX_EXTENSION))
    if logical_channel_extensions is not None:
        request.mux_processor.set_extensions(logical_channel_extensions)
    request.mux_processor.set_quota(8 * 1024)
    return request


def _create_add_channel_request_frame(channel_id, encoding, encoded_handshake):
    # Allow invalid encoding for testing.
    first_byte = ((mux._MUX_OPCODE_ADD_CHANNEL_REQUEST << 5) | encoding)
    payload = (chr(first_byte) +
               mux._encode_channel_id(channel_id) +
               mux._encode_number(len(encoded_handshake)) +
               encoded_handshake)
    return create_binary_frame(
        (mux._encode_channel_id(mux._CONTROL_CHANNEL_ID) + payload), mask=True)


def _create_drop_channel_frame(channel_id, code=None, message=''):
    payload = mux._create_drop_channel(channel_id, code, message)
    return create_binary_frame(
        (mux._encode_channel_id(mux._CONTROL_CHANNEL_ID) + payload), mask=True)


def _create_flow_control_frame(channel_id, replenished_quota):
    payload = mux._create_flow_control(channel_id, replenished_quota)
    return create_binary_frame(
        (mux._encode_channel_id(mux._CONTROL_CHANNEL_ID) + payload), mask=True)


def _create_logical_frame(channel_id, message, opcode=common.OPCODE_BINARY,
                          fin=True, rsv1=False, rsv2=False, rsv3=False,
                          mask=True):
    bits = chr((fin << 7) | (rsv1 << 6) | (rsv2 << 5) | (rsv3 << 4) | opcode)
    payload = mux._encode_channel_id(channel_id) + bits + message
    return create_binary_frame(payload, mask=True)


def _create_request_header(path='/echo', extensions=None):
    headers = (
        'GET %s HTTP/1.1\r\n'
        'Host: server.example.com\r\n'
        'Connection: Upgrade\r\n'
        'Origin: http://example.com\r\n') % path
    if extensions:
        headers += '%s: %s' % (
            common.SEC_WEBSOCKET_EXTENSIONS_HEADER, extensions)
    return headers


class MuxTest(unittest.TestCase):
    """A unittest for mux module."""

    def test_channel_id_decode(self):
        data = '\x00\x01\xbf\xff\xdf\xff\xff\xff\xff\xff\xff'
        parser = mux._MuxFramePayloadParser(data)
        channel_id = parser.read_channel_id()
        self.assertEqual(0, channel_id)
        channel_id = parser.read_channel_id()
        self.assertEqual(1, channel_id)
        channel_id = parser.read_channel_id()
        self.assertEqual(2 ** 14 - 1, channel_id)
        channel_id = parser.read_channel_id()
        self.assertEqual(2 ** 21 - 1, channel_id)
        channel_id = parser.read_channel_id()
        self.assertEqual(2 ** 29 - 1, channel_id)
        self.assertEqual(len(data), parser._read_position)

    def test_channel_id_encode(self):
        encoded = mux._encode_channel_id(0)
        self.assertEqual('\x00', encoded)
        encoded = mux._encode_channel_id(2 ** 14 - 1)
        self.assertEqual('\xbf\xff', encoded)
        encoded = mux._encode_channel_id(2 ** 14)
        self.assertEqual('\xc0@\x00', encoded)
        encoded = mux._encode_channel_id(2 ** 21 - 1)
        self.assertEqual('\xdf\xff\xff', encoded)
        encoded = mux._encode_channel_id(2 ** 21)
        self.assertEqual('\xe0 \x00\x00', encoded)
        encoded = mux._encode_channel_id(2 ** 29 - 1)
        self.assertEqual('\xff\xff\xff\xff', encoded)
        # channel_id is too large
        self.assertRaises(ValueError,
                          mux._encode_channel_id,
                          2 ** 29)

    def test_read_multiple_control_blocks(self):
        # Use AddChannelRequest because it can contain arbitrary length of data
        data = ('\x00\x01\x01a'
                '\x00\x02\x7d%s'
                '\x00\x03\x7e\xff\xff%s'
                '\x00\x04\x7f\x00\x00\x00\x00\x00\x01\x00\x00%s') % (
            'a' * 0x7d, 'b' * 0xffff, 'c' * 0x10000)
        parser = mux._MuxFramePayloadParser(data)
        blocks = list(parser.read_control_blocks())
        self.assertEqual(4, len(blocks))

        self.assertEqual(mux._MUX_OPCODE_ADD_CHANNEL_REQUEST, blocks[0].opcode)
        self.assertEqual(1, blocks[0].channel_id)
        self.assertEqual(1, len(blocks[0].encoded_handshake))

        self.assertEqual(mux._MUX_OPCODE_ADD_CHANNEL_REQUEST, blocks[1].opcode)
        self.assertEqual(2, blocks[1].channel_id)
        self.assertEqual(0x7d, len(blocks[1].encoded_handshake))

        self.assertEqual(mux._MUX_OPCODE_ADD_CHANNEL_REQUEST, blocks[2].opcode)
        self.assertEqual(3, blocks[2].channel_id)
        self.assertEqual(0xffff, len(blocks[2].encoded_handshake))

        self.assertEqual(mux._MUX_OPCODE_ADD_CHANNEL_REQUEST, blocks[3].opcode)
        self.assertEqual(4, blocks[3].channel_id)
        self.assertEqual(0x10000, len(blocks[3].encoded_handshake))

        self.assertEqual(len(data), parser._read_position)

    def test_read_add_channel_request(self):
        data = '\x00\x01\x01a'
        parser = mux._MuxFramePayloadParser(data)
        blocks = list(parser.read_control_blocks())
        self.assertEqual(mux._MUX_OPCODE_ADD_CHANNEL_REQUEST, blocks[0].opcode)
        self.assertEqual(1, blocks[0].channel_id)
        self.assertEqual(1, len(blocks[0].encoded_handshake))

    def test_read_drop_channel(self):
        data = '\x60\x01\x00'
        parser = mux._MuxFramePayloadParser(data)
        blocks = list(parser.read_control_blocks())
        self.assertEqual(1, len(blocks))
        self.assertEqual(1, blocks[0].channel_id)
        self.assertEqual(mux._MUX_OPCODE_DROP_CHANNEL, blocks[0].opcode)
        self.assertEqual(None, blocks[0].drop_code)
        self.assertEqual(0, len(blocks[0].drop_message))

        data = '\x60\x02\x09\x03\xe8Success'
        parser = mux._MuxFramePayloadParser(data)
        blocks = list(parser.read_control_blocks())
        self.assertEqual(1, len(blocks))
        self.assertEqual(2, blocks[0].channel_id)
        self.assertEqual(mux._MUX_OPCODE_DROP_CHANNEL, blocks[0].opcode)
        self.assertEqual(1000, blocks[0].drop_code)
        self.assertEqual('Success', blocks[0].drop_message)

        # Reason is too short.
        data = '\x60\x01\x01\x00'
        parser = mux._MuxFramePayloadParser(data)
        self.assertRaises(mux.PhysicalConnectionError,
                          lambda: list(parser.read_control_blocks()))

    def test_read_flow_control(self):
        data = '\x40\x01\x02'
        parser = mux._MuxFramePayloadParser(data)
        blocks = list(parser.read_control_blocks())
        self.assertEqual(1, len(blocks))
        self.assertEqual(1, blocks[0].channel_id)
        self.assertEqual(mux._MUX_OPCODE_FLOW_CONTROL, blocks[0].opcode)
        self.assertEqual(2, blocks[0].send_quota)

    def test_read_new_channel_slot(self):
        data = '\x80\x01\x02\x02\x03'
        parser = mux._MuxFramePayloadParser(data)
        # TODO(bashi): Implement
        self.assertRaises(mux.PhysicalConnectionError,
                          lambda: list(parser.read_control_blocks()))

    def test_read_invalid_number_field_in_control_block(self):
        # No number field.
        data = ''
        parser = mux._MuxFramePayloadParser(data)
        self.assertRaises(ValueError, parser._read_number)

        # The last two bytes are missing.
        data = '\x7e'
        parser = mux._MuxFramePayloadParser(data)
        self.assertRaises(ValueError, parser._read_number)

        # Missing the last one byte.
        data = '\x7f\x00\x00\x00\x00\x00\x01\x00'
        parser = mux._MuxFramePayloadParser(data)
        self.assertRaises(ValueError, parser._read_number)

        # The length of number field is too large.
        data = '\x7f\xff\xff\xff\xff\xff\xff\xff\xff'
        parser = mux._MuxFramePayloadParser(data)
        self.assertRaises(ValueError, parser._read_number)

        # The msb of the first byte is set.
        data = '\x80'
        parser = mux._MuxFramePayloadParser(data)
        self.assertRaises(ValueError, parser._read_number)

        # Using 3 bytes encoding for 125.
        data = '\x7e\x00\x7d'
        parser = mux._MuxFramePayloadParser(data)
        self.assertRaises(ValueError, parser._read_number)

        # Using 9 bytes encoding for 0xffff
        data = '\x7f\x00\x00\x00\x00\x00\x00\xff\xff'
        parser = mux._MuxFramePayloadParser(data)
        self.assertRaises(ValueError, parser._read_number)

    def test_read_invalid_size_and_contents(self):
        # Only contain number field.
        data = '\x01'
        parser = mux._MuxFramePayloadParser(data)
        self.assertRaises(mux.PhysicalConnectionError,
                          parser._read_size_and_contents)

    def test_create_add_channel_response(self):
        data = mux._create_add_channel_response(channel_id=1,
                                                encoded_handshake='FooBar',
                                                encoding=0,
                                                rejected=False)
        self.assertEqual('\x20\x01\x06FooBar', data)

        data = mux._create_add_channel_response(channel_id=2,
                                                encoded_handshake='Hello',
                                                encoding=1,
                                                rejected=True)
        self.assertEqual('\x31\x02\x05Hello', data)

    def test_create_drop_channel(self):
        data = mux._create_drop_channel(channel_id=1)
        self.assertEqual('\x60\x01\x00', data)

        data = mux._create_drop_channel(channel_id=1,
                                        code=2000,
                                        message='error')
        self.assertEqual('\x60\x01\x07\x07\xd0error', data)

        # reason must be empty if code is None
        self.assertRaises(ValueError,
                          mux._create_drop_channel,
                          1, None, 'FooBar')

    def test_parse_request_text(self):
        request_text = _create_request_header()
        command, path, version, headers = mux._parse_request_text(request_text)
        self.assertEqual('GET', command)
        self.assertEqual('/echo', path)
        self.assertEqual('HTTP/1.1', version)
        self.assertEqual(3, len(headers))
        self.assertEqual('server.example.com', headers['Host'])
        self.assertEqual('http://example.com', headers['Origin'])


class MuxHandlerTest(unittest.TestCase):

    def test_add_channel(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=3, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=3,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Hello'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=3, message='World'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=3, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual([], dispatcher.channel_events[1].messages)
        self.assertEqual(['Hello'], dispatcher.channel_events[2].messages)
        self.assertEqual(['World'], dispatcher.channel_events[3].messages)
        # Channel 2
        messages = request.connection.get_written_messages(2)
        self.assertEqual(1, len(messages))
        self.assertEqual('Hello', messages[0])
        # Channel 3
        messages = request.connection.get_written_messages(3)
        self.assertEqual(1, len(messages))
        self.assertEqual('World', messages[0])
        control_blocks = request.connection.get_written_control_blocks()
        # There should be 8 control blocks:
        #   - 1 NewChannelSlot
        #   - 2 AddChannelResponses for channel id 2 and 3
        #   - 6 FlowControls for channel id 1 (initialize), 'Hello', 'World',
        #     and 3 'Goodbye's
        self.assertEqual(9, len(control_blocks))

    def test_physical_connection_write_failure(self):
        # Use _FailOnWriteConnection.
        request = _create_mock_request(connection=_FailOnWriteConnection())

        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()

        # Let the worker echo back 'Hello'. It causes _FailOnWriteConnection
        # raising an exception.
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Hello'))

        # Let the worker exit. This will be unnecessary when
        # _LogicalConnection.write() is changed to throw an exception if
        # woke up by on_writer_done.
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        # All threads should be done.
        self.assertTrue(mux_handler.wait_until_done(timeout=2))

    def test_send_blocked(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()

        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        # On receiving this 'Hello', the server tries to echo back 'Hello',
        # but it will be blocked since there's no send quota available for the
        # channel 2.
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Hello'))

        # Wait until the worker is blocked due to send quota shortage.
        time.sleep(1)

        # Close the channel 2. The worker should be notified of the end of
        # writer thread and stop waiting for send quota to be replenished.
        drop_channel = _create_drop_channel_frame(channel_id=2)

        request.connection.put_bytes(drop_channel)

        # Make sure the channel 1 is also closed.
        drop_channel = _create_drop_channel_frame(channel_id=1)
        request.connection.put_bytes(drop_channel)

        # All threads should be done.
        self.assertTrue(mux_handler.wait_until_done(timeout=2))

    def test_add_channel_delta_encoding(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        delta = 'GET /echo HTTP/1.1\r\n\r\n'
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=1, encoded_handshake=delta)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Hello'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual(['Hello'], dispatcher.channel_events[2].messages)
        messages = request.connection.get_written_messages(2)
        self.assertEqual(1, len(messages))
        self.assertEqual('Hello', messages[0])

    def test_add_channel_delta_encoding_override(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        # Override Sec-WebSocket-Protocol.
        delta = ('GET /echo HTTP/1.1\r\n'
                 'Sec-WebSocket-Protocol: x-foo\r\n'
                 '\r\n')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=1, encoded_handshake=delta)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Hello'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual(['Hello'], dispatcher.channel_events[2].messages)
        messages = request.connection.get_written_messages(2)
        self.assertEqual(1, len(messages))
        self.assertEqual('Hello', messages[0])
        self.assertEqual('x-foo',
                         dispatcher.channel_events[2].request.ws_protocol)

    def test_add_channel_delta_after_identity(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)
        # Sec-WebSocket-Protocol is different from client's opening handshake
        # of the physical connection.
        # TODO(bashi): Remove Upgrade, Connection, Sec-WebSocket-Key and
        # Sec-WebSocket-Version.
        encoded_handshake = (
            'GET /echo HTTP/1.1\r\n'
            'Host: server.example.com\r\n'
            'Sec-WebSocket-Protocol: x-foo\r\n'
            'Connection: Upgrade\r\n'
            'Origin: http://example.com\r\n'
            '\r\n')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        delta = 'GET /echo HTTP/1.1\r\n\r\n'
        add_channel_request = _create_add_channel_request_frame(
            channel_id=3, encoding=1, encoded_handshake=delta)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=3,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Hello'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=3, message='World'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=3, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual([], dispatcher.channel_events[1].messages)
        self.assertEqual(['Hello'], dispatcher.channel_events[2].messages)
        self.assertEqual(['World'], dispatcher.channel_events[3].messages)
        # Channel 2
        messages = request.connection.get_written_messages(2)
        self.assertEqual(1, len(messages))
        self.assertEqual('Hello', messages[0])
        # Channel 3
        messages = request.connection.get_written_messages(3)
        self.assertEqual(1, len(messages))
        self.assertEqual('World', messages[0])
        # Handshake base should be updated.
        self.assertEqual(
            'x-foo',
            mux_handler._handshake_base._headers['Sec-WebSocket-Protocol'])

    def test_add_channel_delta_remove_header(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)
        # Override handshake delta base.
        encoded_handshake = (
            'GET /echo HTTP/1.1\r\n'
            'Host: server.example.com\r\n'
            'Sec-WebSocket-Protocol: x-foo\r\n'
            'Connection: Upgrade\r\n'
            'Origin: http://example.com\r\n'
            '\r\n')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        # Remove Sec-WebSocket-Protocol header.
        delta = ('GET /echo HTTP/1.1\r\n'
                 'Sec-WebSocket-Protocol:'
                 '\r\n')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=3, encoding=1, encoded_handshake=delta)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=3,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Hello'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=3, message='World'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=3, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual([], dispatcher.channel_events[1].messages)
        self.assertEqual(['Hello'], dispatcher.channel_events[2].messages)
        self.assertEqual(['World'], dispatcher.channel_events[3].messages)
        # Channel 2
        messages = request.connection.get_written_messages(2)
        self.assertEqual(1, len(messages))
        self.assertEqual('Hello', messages[0])
        # Channel 3
        messages = request.connection.get_written_messages(3)
        self.assertEqual(1, len(messages))
        self.assertEqual('World', messages[0])
        self.assertEqual(
            'x-foo',
            dispatcher.channel_events[2].request.ws_protocol)
        self.assertEqual(
            None,
            dispatcher.channel_events[3].request.ws_protocol)

    def test_add_channel_delta_encoding_permessage_compress(self):
        # Enable permessage compress extension on the implicitly opened channel.
        extensions = common.parse_extensions(
            '%s; method=deflate' % common.PERMESSAGE_COMPRESSION_EXTENSION)
        request = _create_mock_request(
            logical_channel_extensions=extensions)
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        delta = 'GET /echo HTTP/1.1\r\n\r\n'
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=1, encoded_handshake=delta)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=20)
        request.connection.put_bytes(flow_control)

        # Send compressed 'Hello' on logical channel 1 and 2.
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message=compressed_hello,
                                  rsv1=True))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message=compressed_hello,
                                  rsv1=True))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual(['Hello'], dispatcher.channel_events[1].messages)
        self.assertEqual(['Hello'], dispatcher.channel_events[2].messages)
        # Written 'Hello's should be compressed.
        messages = request.connection.get_written_messages(1)
        self.assertEqual(1, len(messages))
        self.assertEqual(compressed_hello, messages[0])
        messages = request.connection.get_written_messages(2)
        self.assertEqual(1, len(messages))
        self.assertEqual(compressed_hello, messages[0])

    def test_add_channel_delta_encoding_remove_extensions(self):
        # Enable permessage compress extension on the implicitly opened channel.
        extensions = common.parse_extensions(
            '%s; method=deflate' % common.PERMESSAGE_COMPRESSION_EXTENSION)
        request = _create_mock_request(
            logical_channel_extensions=extensions)
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        # Remove permessage compress extension.
        delta = ('GET /echo HTTP/1.1\r\n'
                 'Sec-WebSocket-Extensions:\r\n'
                 '\r\n')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=1, encoded_handshake=delta)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=20)
        request.connection.put_bytes(flow_control)

        # Send compressed message on logical channel 2. The message should
        # be rejected (since rsv1 is set).
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello = compress.compress('Hello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message=compressed_hello,
                                  rsv1=True))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_NORMAL_CLOSURE, drop_channel.drop_code)
        self.assertEqual(2, drop_channel.channel_id)
        # UnsupportedFrameException should be raised on logical channel 2.
        self.assertTrue(isinstance(dispatcher.channel_events[2].exception,
                                   UnsupportedFrameException))

    def test_add_channel_invalid_encoding(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=3,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_UNKNOWN_REQUEST_ENCODING,
                         drop_channel.drop_code)
        self.assertEqual(common.STATUS_INTERNAL_ENDPOINT_ERROR,
                         request.connection.server_close_code)

    def test_add_channel_incomplete_handshake(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        incomplete_encoded_handshake = 'GET /echo HTTP/1.1'
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=incomplete_encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertTrue(1 in dispatcher.channel_events)
        self.assertTrue(not 2 in dispatcher.channel_events)

    def test_add_channel_duplicate_channel_id(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_CHANNEL_ALREADY_EXISTS,
                         drop_channel.drop_code)
        self.assertEqual(common.STATUS_INTERNAL_ENDPOINT_ERROR,
                         request.connection.server_close_code)

    def test_receive_drop_channel(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        drop_channel = _create_drop_channel_frame(channel_id=2)
        request.connection.put_bytes(drop_channel)

        # Terminate implicitly opened channel.
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        exception = dispatcher.channel_events[2].exception
        self.assertTrue(exception.__class__ == ConnectionTerminatedException)

    def test_receive_ping_frame(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=13)
        request.connection.put_bytes(flow_control)

        ping_frame = _create_logical_frame(channel_id=2,
                                           message='Hello World!',
                                           opcode=common.OPCODE_PING)
        request.connection.put_bytes(ping_frame)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_control_messages(2)
        self.assertEqual(common.OPCODE_PONG, messages[0]['opcode'])
        self.assertEqual('Hello World!', messages[0]['message'])

    def test_receive_fragmented_ping(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=13)
        request.connection.put_bytes(flow_control)

        # Send a ping with message 'Hello world!' in two fragmented frames.
        ping_frame1 = _create_logical_frame(channel_id=2,
                                            message='Hello ',
                                            fin=False,
                                            opcode=common.OPCODE_PING)
        request.connection.put_bytes(ping_frame1)
        ping_frame2 = _create_logical_frame(channel_id=2,
                                            message='World!',
                                            fin=True,
                                            opcode=common.OPCODE_CONTINUATION)
        request.connection.put_bytes(ping_frame2)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_control_messages(2)
        self.assertEqual(common.OPCODE_PONG, messages[0]['opcode'])
        self.assertEqual('Hello World!', messages[0]['message'])

    def test_receive_fragmented_ping_while_receiving_fragmented_message(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=19)
        request.connection.put_bytes(flow_control)

        # Send a fragmented frame of message 'Hello '.
        hello = _create_logical_frame(channel_id=2,
                                      message='Hello ',
                                      fin=False)
        request.connection.put_bytes(hello)

        # Before sending the last fragmented frame of the message, send a
        # fragmented ping.
        ping1 = _create_logical_frame(channel_id=2,
                                      message='Pi',
                                      fin=False,
                                      opcode=common.OPCODE_PING)
        request.connection.put_bytes(ping1)
        ping2 = _create_logical_frame(channel_id=2,
                                      message='ng!',
                                      fin=True,
                                      opcode=common.OPCODE_CONTINUATION)
        request.connection.put_bytes(ping2)

        # Send the last fragmented frame of the message.
        world = _create_logical_frame(channel_id=2,
                                      message='World!',
                                      fin=True,
                                      opcode=common.OPCODE_CONTINUATION)
        request.connection.put_bytes(world)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_messages(2)
        self.assertEqual(['Hello World!'], messages)
        control_messages = request.connection.get_written_control_messages(2)
        self.assertEqual(common.OPCODE_PONG, control_messages[0]['opcode'])
        self.assertEqual('Ping!', control_messages[0]['message'])

    def test_receive_two_ping_while_receiving_fragmented_message(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=25)
        request.connection.put_bytes(flow_control)

        # Send a fragmented frame of message 'Hello '.
        hello = _create_logical_frame(channel_id=2,
                                      message='Hello ',
                                      fin=False)
        request.connection.put_bytes(hello)

        # Before sending the last fragmented frame of the message, send a
        # fragmented ping and a non-fragmented ping.
        ping1 = _create_logical_frame(channel_id=2,
                                      message='Pi',
                                      fin=False,
                                      opcode=common.OPCODE_PING)
        request.connection.put_bytes(ping1)
        ping2 = _create_logical_frame(channel_id=2,
                                      message='ng!',
                                      fin=True,
                                      opcode=common.OPCODE_CONTINUATION)
        request.connection.put_bytes(ping2)
        ping3 = _create_logical_frame(channel_id=2,
                                      message='Pong!',
                                      fin=True,
                                      opcode=common.OPCODE_PING)
        request.connection.put_bytes(ping3)

        # Send the last fragmented frame of the message.
        world = _create_logical_frame(channel_id=2,
                                      message='World!',
                                      fin=True,
                                      opcode=common.OPCODE_CONTINUATION)
        request.connection.put_bytes(world)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_messages(2)
        self.assertEqual(['Hello World!'], messages)
        control_messages = request.connection.get_written_control_messages(2)
        self.assertEqual(common.OPCODE_PONG, control_messages[0]['opcode'])
        self.assertEqual('Ping!', control_messages[0]['message'])
        self.assertEqual(common.OPCODE_PONG, control_messages[1]['opcode'])
        self.assertEqual('Pong!', control_messages[1]['message'])

    def test_receive_message_while_receiving_fragmented_ping(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                replenished_quota=19)
        request.connection.put_bytes(flow_control)

        # Send a fragmented ping.
        ping1 = _create_logical_frame(channel_id=2,
                                      message='Pi',
                                      fin=False,
                                      opcode=common.OPCODE_PING)
        request.connection.put_bytes(ping1)

        # Before sending the last fragmented ping, send a message.
        # The logical channel (2) should be dropped.
        message = _create_logical_frame(channel_id=2,
                                        message='Hello world!',
                                        fin=True)
        request.connection.put_bytes(message)

        # Send the last fragmented frame of the message.
        ping2 = _create_logical_frame(channel_id=2,
                                      message='ng!',
                                      fin=True,
                                      opcode=common.OPCODE_CONTINUATION)
        request.connection.put_bytes(ping2)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(2, drop_channel.channel_id)
        # No message should be sent on channel 2.
        self.assertRaises(KeyError,
                          request.connection.get_written_messages,
                          2)
        self.assertRaises(KeyError,
                          request.connection.get_written_control_messages,
                          2)

    def test_send_ping(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/ping')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_control_messages(2)
        self.assertEqual(common.OPCODE_PING, messages[0]['opcode'])
        self.assertEqual('Ping!', messages[0]['message'])

    def test_send_fragmented_ping(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/ping')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        # Replenish 3 bytes. This isn't enough to send the whole ping frame
        # because the frame will have 5 bytes message('Ping!'). The frame
        # should be fragmented.
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=3)
        request.connection.put_bytes(flow_control)

        # Wait until the worker is blocked due to send quota shortage.
        time.sleep(1)

        # Replenish remaining 2 + 1 bytes (including extra cost).
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=3)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_control_messages(2)
        self.assertEqual(common.OPCODE_PING, messages[0]['opcode'])
        self.assertEqual('Ping!', messages[0]['message'])

    def test_send_fragmented_ping_while_sending_fragmented_message(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(
            path='/ping_while_hello_world')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        # Application will send:
        # - text message 'Hello ' with fin=0
        # - ping with 'Ping!' message
        # - text message 'World!' with fin=1
        # Replenish (6 + 1) + (2 + 1) bytes so that the ping will be
        # fragmented on the logical channel.
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=10)
        request.connection.put_bytes(flow_control)

        time.sleep(1)

        # Replenish remaining 3 + 6 bytes.
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=9)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_messages(2)
        self.assertEqual(['Hello World!'], messages)
        control_messages = request.connection.get_written_control_messages(2)
        self.assertEqual(common.OPCODE_PING, control_messages[0]['opcode'])
        self.assertEqual('Ping!', control_messages[0]['message'])

    def test_send_fragmented_two_ping_while_sending_fragmented_message(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(
            path='/two_ping_while_hello_world')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        # Application will send:
        # - text message 'Hello ' with fin=0
        # - ping with 'Ping!' message
        # - ping with 'Pong!' message
        # - text message 'World!' with fin=1
        # Replenish (6 + 1) + (2 + 1) bytes so that the first ping will be
        # fragmented on the logical channel.
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=10)
        request.connection.put_bytes(flow_control)

        time.sleep(1)

        # Replenish remaining 3 + (5 + 1) + 6 bytes. The second ping won't
        # be fragmented on the logical channel.
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=15)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_messages(2)
        self.assertEqual(['Hello World!'], messages)
        control_messages = request.connection.get_written_control_messages(2)
        self.assertEqual(common.OPCODE_PING, control_messages[0]['opcode'])
        self.assertEqual('Ping!', control_messages[0]['message'])
        self.assertEqual(common.OPCODE_PING, control_messages[1]['opcode'])
        self.assertEqual('Pong!', control_messages[1]['message'])

    def test_send_drop_channel(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()

        # DropChannel for channel id 1 which doesn't have reason.
        frame = create_binary_frame('\x00\x60\x01\x00', mask=True)
        request.connection.put_bytes(frame)

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_ACKNOWLEDGED,
                         drop_channel.drop_code)
        self.assertEqual(1, drop_channel.channel_id)

    def test_two_flow_control(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        # Replenish 5 bytes.
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=5)
        request.connection.put_bytes(flow_control)

        # Send 10 bytes. The server will try echo back 10 bytes.
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='HelloWorld'))

        # Replenish 5 + 1 (per-message extra cost) bytes.
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_messages(2)
        self.assertEqual(['HelloWorld'], messages)
        received_flow_controls = [
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_FLOW_CONTROL and b.channel_id == 2]
        # Replenishment for 'HelloWorld' + 1
        self.assertEqual(11, received_flow_controls[0].send_quota)
        # Replenishment for 'Goodbye' + 1
        self.assertEqual(8, received_flow_controls[1].send_quota)

    def test_no_send_quota_on_server(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='HelloWorld'))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        # Just wait for 1 sec so that the server attempts to echo back
        # 'HelloWorld'.
        self.assertFalse(mux_handler.wait_until_done(timeout=1))

        # No message should be sent on channel 2.
        self.assertRaises(KeyError,
                          request.connection.get_written_messages,
                          2)

    def test_no_send_quota_on_server_for_permessage_extra_cost(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Hello'))
        # Replenish only len('World') bytes.
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=5)
        request.connection.put_bytes(flow_control)
        # Server should not callback for this message.
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='World'))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        # Just wait for 1 sec so that the server attempts to echo back
        # 'World'.
        self.assertFalse(mux_handler.wait_until_done(timeout=1))

        # Only one message should be sent on channel 2.
        messages = request.connection.get_written_messages(2)
        self.assertEqual(['Hello'], messages)

    def test_quota_violation_by_client(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS, 0)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='HelloWorld'))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        control_blocks = request.connection.get_written_control_blocks()
        self.assertEqual(5, len(control_blocks))
        drop_channel = next(
            b for b in control_blocks
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_SEND_QUOTA_VIOLATION,
                         drop_channel.drop_code)

    def test_consume_quota_empty_message(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        # Client has 1 byte quota.
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS, 1)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=2)
        request.connection.put_bytes(flow_control)
        # Send an empty message. Pywebsocket always replenishes 1 byte quota
        # for empty message
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message=''))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        # This message violates quota on channel id 2.
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual(1, len(dispatcher.channel_events[2].messages))
        self.assertEqual('', dispatcher.channel_events[2].messages[0])

        received_flow_controls = [
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_FLOW_CONTROL and b.channel_id == 2]
        self.assertEqual(1, len(received_flow_controls))
        self.assertEqual(1, received_flow_controls[0].send_quota)

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(2, drop_channel.channel_id)
        self.assertEqual(mux._DROP_CODE_SEND_QUOTA_VIOLATION,
                         drop_channel.drop_code)

    def test_consume_quota_fragmented_message(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        # Client has len('Hello') + len('Goodbye') + 2 bytes quota.
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS, 14)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='He', fin=False,
                                  opcode=common.OPCODE_TEXT))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='llo', fin=True,
                                  opcode=common.OPCODE_CONTINUATION))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_messages(2)
        self.assertEqual(['Hello'], messages)

    def test_fragmented_control_message(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/ping')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        # Replenish total 6 bytes in 3 FlowControls.
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=1)
        request.connection.put_bytes(flow_control)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=2)
        request.connection.put_bytes(flow_control)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=3)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        messages = request.connection.get_written_control_messages(2)
        self.assertEqual(common.OPCODE_PING, messages[0]['opcode'])
        self.assertEqual('Ping!', messages[0]['message'])

    def test_channel_slot_violation_by_client(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(slots=1,
                                      send_quota=mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)
        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Hello'))

        # This request should be rejected.
        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=3, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)
        flow_control = _create_flow_control_frame(channel_id=3,
                                                  replenished_quota=6)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=3, message='Hello'))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual([], dispatcher.channel_events[1].messages)
        self.assertEqual(['Hello'], dispatcher.channel_events[2].messages)
        self.assertFalse(dispatcher.channel_events.has_key(3))
        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(3, drop_channel.channel_id)
        self.assertEqual(mux._DROP_CODE_NEW_CHANNEL_SLOT_VIOLATION,
                         drop_channel.drop_code)

    def test_quota_overflow_by_client(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(slots=1,
                                      send_quota=mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)
        # Replenish 0x7FFFFFFFFFFFFFFF bytes twice.
        flow_control = _create_flow_control_frame(
            channel_id=2,
            replenished_quota=0x7FFFFFFFFFFFFFFF)
        request.connection.put_bytes(flow_control)
        request.connection.put_bytes(flow_control)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(2, drop_channel.channel_id)
        self.assertEqual(mux._DROP_CODE_SEND_QUOTA_OVERFLOW,
                         drop_channel.drop_code)

    def test_invalid_encapsulated_message(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()

        first_byte = (mux._MUX_OPCODE_ADD_CHANNEL_REQUEST << 5)
        block = (chr(first_byte) +
                 mux._encode_channel_id(1) +
                 mux._encode_number(0))
        payload = mux._encode_channel_id(mux._CONTROL_CHANNEL_ID) + block
        text_frame = create_binary_frame(payload, opcode=common.OPCODE_TEXT,
                                         mask=True)
        request.connection.put_bytes(text_frame)

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_INVALID_ENCAPSULATING_MESSAGE,
                         drop_channel.drop_code)
        self.assertEqual(common.STATUS_INTERNAL_ENDPOINT_ERROR,
                         request.connection.server_close_code)

    def test_channel_id_truncated(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()

        # The last byte of the channel id is missing.
        frame = create_binary_frame('\x80', mask=True)
        request.connection.put_bytes(frame)

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_CHANNEL_ID_TRUNCATED,
                         drop_channel.drop_code)
        self.assertEqual(common.STATUS_INTERNAL_ENDPOINT_ERROR,
                         request.connection.server_close_code)

    def test_inner_frame_truncated(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()

        # Just contain channel id 1.
        frame = create_binary_frame('\x01', mask=True)
        request.connection.put_bytes(frame)

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_ENCAPSULATED_FRAME_IS_TRUNCATED,
                         drop_channel.drop_code)
        self.assertEqual(common.STATUS_INTERNAL_ENDPOINT_ERROR,
                         request.connection.server_close_code)

    def test_unknown_mux_opcode(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()

        # Undefined opcode 5
        frame = create_binary_frame('\x00\xa0', mask=True)
        request.connection.put_bytes(frame)

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_UNKNOWN_MUX_OPCODE,
                         drop_channel.drop_code)
        self.assertEqual(common.STATUS_INTERNAL_ENDPOINT_ERROR,
                         request.connection.server_close_code)

    def test_invalid_mux_control_block(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()

        # DropChannel contains 1 byte reason
        frame = create_binary_frame('\x00\x60\x00\x01\x00', mask=True)
        request.connection.put_bytes(frame)

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channel = next(
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL)
        self.assertEqual(mux._DROP_CODE_INVALID_MUX_CONTROL_BLOCK,
                         drop_channel.drop_code)
        self.assertEqual(common.STATUS_INTERNAL_ENDPOINT_ERROR,
                         request.connection.server_close_code)

    def test_permessage_compress(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        # Enable permessage compress extension on logical channel 2.
        extensions = '%s; method=deflate' % (
            common.PERMESSAGE_COMPRESSION_EXTENSION)
        encoded_handshake = _create_request_header(path='/echo',
                                                   extensions=extensions)
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        flow_control = _create_flow_control_frame(channel_id=2,
                                                  replenished_quota=20)
        request.connection.put_bytes(flow_control)

        # Send compressed 'Hello' twice.
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello1 = compress.compress('Hello')
        compressed_hello1 += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello1 = compressed_hello1[:-4]
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message=compressed_hello1,
                                  rsv1=True))
        compressed_hello2 = compress.compress('Hello')
        compressed_hello2 += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello2 = compressed_hello2[:-4]
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message=compressed_hello2,
                                  rsv1=True))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=2, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual(['Hello', 'Hello'],
                         dispatcher.channel_events[2].messages)
        # Written 'Hello's should be compressed.
        messages = request.connection.get_written_messages(2)
        self.assertEqual(2, len(messages))
        self.assertEqual(compressed_hello1, messages[0])
        self.assertEqual(compressed_hello2, messages[1])


    def test_permessage_compress_fragmented_message(self):
        extensions = common.parse_extensions(
            '%s; method=deflate' % common.PERMESSAGE_COMPRESSION_EXTENSION)
        request = _create_mock_request(
            logical_channel_extensions=extensions)
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        # Send compressed 'HelloHelloHello' as fragmented message.
        compress = zlib.compressobj(
            zlib.Z_DEFAULT_COMPRESSION, zlib.DEFLATED, -zlib.MAX_WBITS)
        compressed_hello = compress.compress('HelloHelloHello')
        compressed_hello += compress.flush(zlib.Z_SYNC_FLUSH)
        compressed_hello = compressed_hello[:-4]

        m = len(compressed_hello) / 2
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1,
                                  message=compressed_hello[:m],
                                  fin=False, rsv1=True,
                                  opcode=common.OPCODE_TEXT))
        request.connection.put_bytes(
            _create_logical_frame(channel_id=1,
                                  message=compressed_hello[m:],
                                  fin=True, rsv1=False,
                                  opcode=common.OPCODE_CONTINUATION))

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        self.assertEqual(['HelloHelloHello'],
                         dispatcher.channel_events[1].messages)
        messages = request.connection.get_written_messages(1)
        self.assertEqual(1, len(messages))
        self.assertEqual(compressed_hello, messages[0])

    def test_receive_bad_fragmented_message(self):
        request = _create_mock_request()
        dispatcher = _MuxMockDispatcher()
        mux_handler = mux._MuxHandler(request, dispatcher)
        mux_handler.start()
        mux_handler.add_channel_slots(mux._INITIAL_NUMBER_OF_CHANNEL_SLOTS,
                                      mux._INITIAL_QUOTA_FOR_CLIENT)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=2, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        # Send a frame with fin=False, and then send a frame with
        # opcode=TEXT (not CONTINUATION). Logical channel 2 should be dropped.
        frame1 = _create_logical_frame(channel_id=2,
                                       message='Hello ',
                                       fin=False,
                                       opcode=common.OPCODE_TEXT)
        request.connection.put_bytes(frame1)
        frame2 = _create_logical_frame(channel_id=2,
                                       message='World!',
                                       fin=True,
                                       opcode=common.OPCODE_TEXT)
        request.connection.put_bytes(frame2)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=3, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        # Send a frame with opcode=CONTINUATION without a preceding frame
        # the fin of which is not set. Logical channel 3 should be dropped.
        frame3 = _create_logical_frame(channel_id=3,
                                       message='Hello',
                                       fin=True,
                                       opcode=common.OPCODE_CONTINUATION)
        request.connection.put_bytes(frame3)

        encoded_handshake = _create_request_header(path='/echo')
        add_channel_request = _create_add_channel_request_frame(
            channel_id=4, encoding=0,
            encoded_handshake=encoded_handshake)
        request.connection.put_bytes(add_channel_request)

        # Send a frame with opcode=PING and fin=False, and then send a frame
        # with opcode=TEXT (not CONTINUATION). Logical channel 4 should be
        # dropped.
        frame4 = _create_logical_frame(channel_id=4,
                                       message='Ping',
                                       fin=False,
                                       opcode=common.OPCODE_PING)
        request.connection.put_bytes(frame4)
        frame5 = _create_logical_frame(channel_id=4,
                                       message='Hello',
                                       fin=True,
                                       opcode=common.OPCODE_TEXT)
        request.connection.put_bytes(frame5)

        request.connection.put_bytes(
            _create_logical_frame(channel_id=1, message='Goodbye'))

        self.assertTrue(mux_handler.wait_until_done(timeout=2))

        drop_channels = [
            b for b in request.connection.get_written_control_blocks()
            if b.opcode == mux._MUX_OPCODE_DROP_CHANNEL]
        self.assertEqual(3, len(drop_channels))
        for d in drop_channels:
            self.assertEqual(mux._DROP_CODE_BAD_FRAGMENTATION,
                             d.drop_code)


if __name__ == '__main__':
    unittest.main()


# vi:sts=4 sw=4 et
