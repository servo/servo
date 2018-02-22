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


"""WebSocket client utility for testing mux extension.

This code should be independent from mod_pywebsocket. See the comment of
client_for_testing.py.

NOTE: This code is far from robust like client_for_testing.py.
"""



import Queue
import base64
import collections
import email
import email.parser
import logging
import math
import os
import random
import socket
import struct
import threading

from mod_pywebsocket import util

from test import client_for_testing


_CONTROL_CHANNEL_ID = 0
_DEFAULT_CHANNEL_ID = 1

_MUX_OPCODE_ADD_CHANNEL_REQUEST = 0
_MUX_OPCODE_ADD_CHANNEL_RESPONSE = 1
_MUX_OPCODE_FLOW_CONTROL = 2
_MUX_OPCODE_DROP_CHANNEL = 3
_MUX_OPCODE_NEW_CHANNEL_SLOT = 4


class _ControlBlock:
    def __init__(self, opcode):
        self.opcode = opcode


def _parse_handshake_response(response):
    status_line, header_lines = response.split('\r\n', 1)

    words = status_line.split(' ')
    if len(words) < 3:
        raise ValueError('Bad Status-Line syntax %r' % status_line)
    [version, response_code] = words[:2]
    if version != 'HTTP/1.1':
        raise ValueError('Bad response version %r' % version)

    if response_code != '101':
        raise ValueError('Bad response code %r ' % response_code)
    headers = email.parser.Parser().parsestr(header_lines)
    return headers


def _parse_channel_id(data, offset=0):
    length = len(data)
    remaining = length - offset

    if remaining <= 0:
        raise Exception('No channel id found')

    channel_id = ord(data[offset])
    channel_id_length = 1
    if channel_id & 0xe0 == 0xe0:
        if remaining < 4:
            raise Exception('Invalid channel id format')
        channel_id = struct.unpack('!L',
                                   data[offset:offset+4])[0] & 0x1fffffff
        channel_id_length = 4
    elif channel_id & 0xc0 == 0xc0:
        if remaining < 3:
            raise Exception('Invalid channel id format')
        channel_id = (((channel_id & 0x1f) << 16) +
                      struct.unpack('!H', data[offset+1:offset+3])[0])
        channel_id_length = 3
    elif channel_id & 0x80 == 0x80:
        if remaining < 2:
            raise Exception('Invalid channel id format')
        channel_id = struct.unpack('!H', data[offset:offset+2])[0] & 0x3fff
        channel_id_length = 2

    return channel_id, channel_id_length


def _parse_number(data, offset=0):
    first_byte = ord(data[offset])
    if (first_byte & 0x80) != 0:
        raise Exception('The MSB of number field must be unset')
    first_byte = first_byte & 0x7f
    if first_byte == 127:
        if offset + 9 > len(data):
            raise Exception('Invalid number')
        return struct.unpack('!Q', data[offset+1:offset+9])[0], 9
    if first_byte == 126:
        if offset + 3 > len(data):
            raise Exception('Invalid number')
        return struct.unpack('!H', data[offset+1:offset+3])[0], 3
    return first_byte, 1


def _parse_size_and_contents(data, offset=0):
    size, advance = _parse_number(data, offset)
    start_position = offset + advance
    end_position = start_position + size
    if len(data) < end_position:
        raise Exception('Invalid size of control block (%d < %d)' % (
                len(data), end_position))
    return data[start_position:end_position], size + advance


def _parse_control_blocks(data):
    blocks = []
    length = len(data)
    pos = 0

    while pos < length:
        first_byte = ord(data[pos])
        pos += 1
        opcode = (first_byte >> 5) & 0x7
        block = _ControlBlock(opcode)

        # TODO(bashi): Support more opcode
        if opcode == _MUX_OPCODE_ADD_CHANNEL_RESPONSE:
            block.encode = first_byte & 3
            block.rejected = (first_byte >> 4) & 1

            channel_id, advance = _parse_channel_id(data, pos)
            block.channel_id = channel_id
            pos += advance

            encoded_handshake, advance = _parse_size_and_contents(data, pos)
            block.encoded_handshake = encoded_handshake
            pos += advance
            blocks.append(block)
        elif opcode == _MUX_OPCODE_DROP_CHANNEL:
            block.mux_error = (first_byte >> 4) & 1

            channel_id, advance = _parse_channel_id(data, pos)
            block.channel_id = channel_id
            pos += advance

            reason, advance = _parse_size_and_contents(data, pos)
            if len(reason) == 0:
                block.drop_code = None
                block.drop_message = ''
            elif len(reason) >= 2:
                block.drop_code = struct.unpack('!H', reason[:2])[0]
                block.drop_message = reason[2:]
            else:
                raise Exception('Invalid DropChannel')
            pos += advance
            blocks.append(block)
        elif opcode == _MUX_OPCODE_FLOW_CONTROL:
            channel_id, advance = _parse_channel_id(data, pos)
            block.channel_id = channel_id
            pos += advance
            send_quota, advance = _parse_number(data, pos)
            block.send_quota = send_quota
            pos += advance
            blocks.append(block)
        elif opcode == _MUX_OPCODE_NEW_CHANNEL_SLOT:
            fallback = first_byte & 1
            slots, advance = _parse_number(data, pos)
            pos += advance
            send_quota, advance = _parse_number(data, pos)
            pos += advance
            if fallback == 1 and (slots != 0 or send_quota != 0):
                raise Exception('slots and send_quota must be zero if F bit '
                                'is set')
            block.fallback = fallback
            block.slots = slots
            block.send_quota = send_quota
            blocks.append(block)
        else:
            raise Exception(
                'Unsupported mux opcode %d received' % opcode)

    return blocks


def _encode_channel_id(channel_id):
    if channel_id < 0:
        raise ValueError('Channel id %d must not be negative' % channel_id)

    if channel_id < 2 ** 7:
        return chr(channel_id)
    if channel_id < 2 ** 14:
        return struct.pack('!H', 0x8000 + channel_id)
    if channel_id < 2 ** 21:
        first = chr(0xc0 + (channel_id >> 16))
        return first + struct.pack('!H', channel_id & 0xffff)
    if channel_id < 2 ** 29:
        return struct.pack('!L', 0xe0000000 + channel_id)

    raise ValueError('Channel id %d is too large' % channel_id)


def _encode_number(number):
    if number <= 125:
        return chr(number)
    elif number < (1 << 16):
        return chr(0x7e) + struct.pack('!H', number)
    elif number < (1 << 63):
        return chr(0x7f) + struct.pack('!Q', number)
    else:
        raise Exception('Invalid number')


def _create_add_channel_request(channel_id, encoded_handshake,
                                encoding=0):
    length = len(encoded_handshake)
    handshake_length = _encode_number(length)

    first_byte = (_MUX_OPCODE_ADD_CHANNEL_REQUEST << 5) | encoding
    return (chr(first_byte) + _encode_channel_id(channel_id) +
            handshake_length + encoded_handshake)


def _create_flow_control(channel_id, replenished_quota):
    first_byte = (_MUX_OPCODE_FLOW_CONTROL << 5)
    return (chr(first_byte) + _encode_channel_id(channel_id) +
            _encode_number(replenished_quota))


class _MuxReaderThread(threading.Thread):
    """Mux reader thread.

    Reads frames and passes them to the mux client. This thread accesses
    private functions/variables of the mux client.
    """

    def __init__(self, mux):
        threading.Thread.__init__(self)
        self.setDaemon(True)
        self._mux = mux
        self._stop_requested = False

    def _receive_message(self):
        first_opcode = None
        pending_payload = []
        while not self._stop_requested:
            fin, rsv1, rsv2, rsv3, opcode, payload_length = (
                client_for_testing.read_frame_header(self._mux._socket))

            if not first_opcode:
                if opcode == client_for_testing.OPCODE_TEXT:
                    raise Exception('Received a text message on physical '
                                    'connection')
                if opcode == client_for_testing.OPCODE_CONTINUATION:
                    raise Exception('Received an intermediate frame but '
                                    'fragmentation was not started')
                if (opcode == client_for_testing.OPCODE_BINARY or
                    opcode == client_for_testing.OPCODE_PONG or
                    opcode == client_for_testing.OPCODE_PONG or
                    opcode == client_for_testing.OPCODE_CLOSE):
                    first_opcode = opcode
                else:
                    raise Exception('Received an undefined opcode frame: %d' %
                                    opcode)

            elif opcode != client_for_testing.OPCODE_CONTINUATION:
                raise Exception('Received a new opcode before '
                                'terminating fragmentation')

            payload = client_for_testing.receive_bytes(
                self._mux._socket, payload_length)

            if self._mux._incoming_frame_filter is not None:
                payload = self._mux._incoming_frame_filter.filter(payload)

            pending_payload.append(payload)

            if fin:
                break

        if self._stop_requested:
            return None, None

        message = ''.join(pending_payload)
        return first_opcode, message

    def request_stop(self):
        self._stop_requested = True

    def run(self):
        try:
            while not self._stop_requested:
                # opcode is OPCODE_BINARY or control opcodes when a message
                # is succesfully received.
                opcode, message = self._receive_message()
                if not opcode:
                    return
                if opcode == client_for_testing.OPCODE_BINARY:
                    channel_id, advance = _parse_channel_id(message)
                    self._mux._dispatch_frame(channel_id, message[advance:])
                else:
                    self._mux._process_control_message(opcode, message)
        finally:
            self._mux._notify_reader_done()


class _InnerFrame(object):
    def __init__(self, fin, rsv1, rsv2, rsv3, opcode, payload):
        self.fin = fin
        self.rsv1 = rsv1
        self.rsv2 = rsv2
        self.rsv3 = rsv3
        self.opcode = opcode
        self.payload = payload


class _LogicalChannelData(object):
    def __init__(self):
        self.queue = Queue.Queue()
        self.send_quota = 0
        self.receive_quota = 0


class MuxClient(object):
    """WebSocket mux client.

    Note that this class is NOT thread-safe. Do not access an instance of this
    class from multiple threads at a same time.
    """

    def __init__(self, options):
        self._logger = util.get_class_logger(self)

        self._options = options
        self._options.enable_mux()
        self._stream = None
        self._socket = None
        self._handshake = client_for_testing.WebSocketHandshake(self._options)
        self._incoming_frame_filter = None
        self._outgoing_frame_filter = None

        self._is_active = False
        self._read_thread = None
        self._control_blocks_condition = threading.Condition()
        self._control_blocks = []
        self._channel_slots = collections.deque()
        self._logical_channels_condition = threading.Condition();
        self._logical_channels = {}
        self._timeout = 2
        self._physical_connection_close_event = None
        self._physical_connection_close_message = None

    def _parse_inner_frame(self, data):
        if len(data) == 0:
            raise Exception('Invalid encapsulated frame received')

        first_byte = ord(data[0])
        fin = (first_byte << 7) & 1
        rsv1 = (first_byte << 6) & 1
        rsv2 = (first_byte << 5) & 1
        rsv3 = (first_byte << 4) & 1
        opcode = first_byte & 0xf

        if self._outgoing_frame_filter:
            payload = self._outgoing_frame_filter.filter(
                data[1:])
        else:
            payload = data[1:]

        return _InnerFrame(fin, rsv1, rsv2, rsv3, opcode, payload)

    def _process_mux_control_blocks(self):
        for block in self._control_blocks:
            if block.opcode == _MUX_OPCODE_ADD_CHANNEL_RESPONSE:
                # AddChannelResponse will be handled in add_channel().
                continue
            elif block.opcode == _MUX_OPCODE_FLOW_CONTROL:
                try:
                    self._logical_channels_condition.acquire()
                    if not block.channel_id in self._logical_channels:
                        raise Exception('Invalid flow control received for '
                                        'channel id %d' % block.channel_id)
                    self._logical_channels[block.channel_id].send_quota += (
                        block.send_quota)
                    self._logical_channels_condition.notify()
                finally:
                    self._logical_channels_condition.release()
            elif block.opcode == _MUX_OPCODE_NEW_CHANNEL_SLOT:
                self._channel_slots.extend([block.send_quota] * block.slots)

    def _dispatch_frame(self, channel_id, payload):
        if channel_id == _CONTROL_CHANNEL_ID:
            try:
                self._control_blocks_condition.acquire()
                self._control_blocks += _parse_control_blocks(payload)
                self._process_mux_control_blocks()
                self._control_blocks_condition.notify()
            finally:
                self._control_blocks_condition.release()
        else:
            try:
                self._logical_channels_condition.acquire()
                if not channel_id in self._logical_channels:
                    raise Exception('Received logical frame on channel id '
                                    '%d, which is not established' %
                                    channel_id)

                inner_frame = self._parse_inner_frame(payload)
                self._logical_channels[channel_id].receive_quota -= (
                        len(inner_frame.payload))
                if self._logical_channels[channel_id].receive_quota < 0:
                    raise Exception('The server violates quota on '
                                    'channel id %d' % channel_id)
            finally:
                self._logical_channels_condition.release()
            self._logical_channels[channel_id].queue.put(inner_frame)

    def _process_control_message(self, opcode, message):
        # Ping/Pong are not supported.
        if opcode == client_for_testing.OPCODE_CLOSE:
            self._physical_connection_close_message = message
            if self._is_active:
                self._stream.send_close(
                    code=client_for_testing.STATUS_NORMAL_CLOSURE, reason='')
                self._read_thread.request_stop()

            if self._physical_connection_close_event:
                self._physical_connection_close_event.set()

    def _notify_reader_done(self):
        self._logger.debug('Read thread terminated.')
        self.close_socket()

    def _assert_channel_slot_available(self):
        try:
            self._control_blocks_condition.acquire()
            if len(self._channel_slots) == 0:
                # Wait once
                self._control_blocks_condition.wait(timeout=self._timeout)
        finally:
            self._control_blocks_condition.release()

        if len(self._channel_slots) == 0:
            raise Exception('Failed to receive NewChannelSlot')

    def _assert_send_quota_available(self, channel_id):
        try:
            self._logical_channels_condition.acquire()
            if self._logical_channels[channel_id].send_quota == 0:
                # Wait once
                self._logical_channels_condition.wait(timeout=self._timeout)
        finally:
            self._logical_channels_condition.release()

        if self._logical_channels[channel_id].send_quota == 0:
            raise Exception('Failed to receive FlowControl for channel id %d' %
                            channel_id)

    def connect(self):
        self._socket = client_for_testing.connect_socket_with_retry(
                self._options.server_host,
                self._options.server_port,
                self._options.socket_timeout,
                self._options.use_tls)

        self._handshake.handshake(self._socket)
        self._stream = client_for_testing.WebSocketStream(
            self._socket, self._handshake)

        self._logical_channels[_DEFAULT_CHANNEL_ID] = _LogicalChannelData()

        self._read_thread = _MuxReaderThread(self)
        self._read_thread.start()

        self._assert_channel_slot_available()
        self._assert_send_quota_available(_DEFAULT_CHANNEL_ID)

        self._is_active = True
        self._logger.info('Connection established')

    def add_channel(self, channel_id, options):
        if not self._is_active:
            raise Exception('Mux client is not active')

        if channel_id in self._logical_channels:
            raise Exception('Channel id %d already exists' % channel_id)

        try:
            send_quota = self._channel_slots.popleft()
        except IndexError, e:
            raise Exception('No channel slots: %r' % e)

        # Create AddChannel request
        request_line = 'GET %s HTTP/1.1\r\n' % options.resource
        fields = []
        if options.server_port == client_for_testing.DEFAULT_PORT:
            fields.append('Host: %s\r\n' % options.server_host.lower())
        else:
            fields.append('Host: %s:%d\r\n' % (options.server_host.lower(),
                                               options.server_port))
        fields.append('Origin: %s\r\n' % options.origin.lower())
        fields.append('Connection: Upgrade\r\n')

        if len(options.extensions) > 0:
            fields.append('Sec-WebSocket-Extensions: %s\r\n' %
                          ', '.join(options.extensions))

        handshake = request_line + ''.join(fields) + '\r\n'
        add_channel_request = _create_add_channel_request(
            channel_id, handshake)
        payload = _encode_channel_id(_CONTROL_CHANNEL_ID) + add_channel_request
        self._stream.send_binary(payload)

        # Wait AddChannelResponse
        self._logger.debug('Waiting AddChannelResponse for the request...')
        response = None
        try:
            self._control_blocks_condition.acquire()
            while True:
                for block in self._control_blocks:
                    if block.opcode != _MUX_OPCODE_ADD_CHANNEL_RESPONSE:
                        continue
                    if block.channel_id == channel_id:
                        response = block
                        self._control_blocks.remove(response)
                        break
                if response:
                    break
                self._control_blocks_condition.wait(self._timeout)
                if not self._is_active:
                    raise Exception('AddChannelRequest timed out')
        finally:
            self._control_blocks_condition.release()

        # Validate AddChannelResponse
        if response.rejected:
            raise Exception('The server rejected AddChannelRequest')

        fields = _parse_handshake_response(response.encoded_handshake)

        # Should we reject when Upgrade, Connection, or Sec-WebSocket-Accept
        # headers exist?

        self._logical_channels_condition.acquire()
        self._logical_channels[channel_id] = _LogicalChannelData()
        self._logical_channels[channel_id].send_quota = send_quota
        self._logical_channels_condition.release()

        self._logger.debug('Logical channel %d established' % channel_id)

    def _check_logical_channel_is_opened(self, channel_id):
        if not self._is_active:
            raise Exception('Mux client is not active')

        if not channel_id in self._logical_channels:
            raise Exception('Logical channel %d is not established.')

    def drop_channel(self, channel_id):
        # TODO(bashi): Implement
        pass

    def send_flow_control(self, channel_id, replenished_quota):
        self._check_logical_channel_is_opened(channel_id)
        flow_control = _create_flow_control(channel_id, replenished_quota)
        payload = _encode_channel_id(_CONTROL_CHANNEL_ID) + flow_control
        # Replenish receive quota
        try:
            self._logical_channels_condition.acquire()
            self._logical_channels[channel_id].receive_quota += (
                replenished_quota)
        finally:
            self._logical_channels_condition.release()
        self._stream.send_binary(payload)

    def send_message(self, channel_id, message, end=True, binary=False):
        self._check_logical_channel_is_opened(channel_id)

        if binary:
            first_byte = (end << 7) | client_for_testing.OPCODE_BINARY
        else:
            first_byte = (end << 7) | client_for_testing.OPCODE_TEXT
            message = message.encode('utf-8')

        try:
            self._logical_channels_condition.acquire()
            if self._logical_channels[channel_id].send_quota < len(message):
                raise Exception('Send quota violation: %d < %d' % (
                        self._logical_channels[channel_id].send_quota,
                        len(message)))

            self._logical_channels[channel_id].send_quota -= len(message)
        finally:
            self._logical_channels_condition.release()
        payload = _encode_channel_id(channel_id) + chr(first_byte) + message
        self._stream.send_binary(payload)

    def assert_receive(self, channel_id, payload, binary=False):
        self._check_logical_channel_is_opened(channel_id)

        try:
            inner_frame = self._logical_channels[channel_id].queue.get(
                timeout=self._timeout)
        except Queue.Empty, e:
            raise Exception('Cannot receive message from channel id %d' %
                            channel_id)

        if binary:
            opcode = client_for_testing.OPCODE_BINARY
        else:
            opcode = client_for_testing.OPCODE_TEXT

        if inner_frame.opcode != opcode:
            raise Exception('Unexpected opcode received (%r != %r)' %
                            (expected_opcode, inner_frame.opcode))

        if inner_frame.payload != payload:
            raise Exception('Unexpected payload received')

    def send_close(self, channel_id, code=None, reason=''):
        self._check_logical_channel_is_opened(channel_id)

        if code is not None:
            body = struct.pack('!H', code) + reason.encode('utf-8')
        else:
            body = ''

        first_byte = (1 << 7) | client_for_testing.OPCODE_CLOSE
        payload = _encode_channel_id(channel_id) + chr(first_byte) + body
        self._stream.send_binary(payload)

    def assert_receive_close(self, channel_id):
        self._check_logical_channel_is_opened(channel_id)

        try:
            inner_frame = self._logical_channels[channel_id].queue.get(
                timeout=self._timeout)
        except Queue.Empty, e:
            raise Exception('Cannot receive message from channel id %d' %
                            channel_id)
        if inner_frame.opcode != client_for_testing.OPCODE_CLOSE:
            raise Exception('Didn\'t receive close frame')

    def send_physical_connection_close(self, code=None, reason=''):
        self._physical_connection_close_event = threading.Event()
        self._stream.send_close(code, reason)

    # This method can be used only after calling
    # send_physical_connection_close().
    def assert_physical_connection_receive_close(
        self, code=client_for_testing.STATUS_NORMAL_CLOSURE, reason=''):
        self._physical_connection_close_event.wait(timeout=self._timeout)
        if (not self._physical_connection_close_event.isSet() or
            not self._physical_connection_close_message):
            raise Exception('Didn\'t receive closing handshake')

    def close_socket(self):
        self._is_active = False
        self._socket.close()
