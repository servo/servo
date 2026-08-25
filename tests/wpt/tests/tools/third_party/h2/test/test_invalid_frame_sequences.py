# -*- coding: utf-8 -*-
"""
test_invalid_frame_sequences.py
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This module contains tests that use invalid frame sequences, and validates that
they fail appropriately.
"""
import pytest

import h2.config
import h2.connection
import h2.errors
import h2.events
import h2.exceptions


class TestInvalidFrameSequences(object):
    """
    Invalid frame sequences, either sent or received, cause ProtocolErrors to
    be thrown.
    """
    example_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'GET'),
    ]
    example_response_headers = [
        (':status', '200'),
        ('server', 'fake-serv/0.1.0')
    ]
    server_config = h2.config.H2Configuration(client_side=False)
    client_config = h2.config.H2Configuration(client_side=True)

    def test_cannot_send_on_closed_stream(self):
        """
        When we've closed a stream locally, we cannot send further data.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers, end_stream=True)

        with pytest.raises(h2.exceptions.ProtocolError):
            c.send_data(1, b'some data')

    def test_missing_preamble_errors(self):
        """
        Server side connections require the preamble.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        encoded_headers_frame = (
            b'\x00\x00\r\x01\x04\x00\x00\x00\x01'
            b'A\x88/\x91\xd3]\x05\\\x87\xa7\x84\x87\x82'
        )

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(encoded_headers_frame)

    def test_server_connections_reject_even_streams(self, frame_factory):
        """
        Servers do not allow clients to initiate even-numbered streams.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            self.example_request_headers, stream_id=2
        )

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

    def test_clients_reject_odd_stream_pushes(self, frame_factory):
        """
        Clients do not allow servers to push odd numbered streams.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers, end_stream=True)

        f = frame_factory.build_push_promise_frame(
            stream_id=1,
            headers=self.example_request_headers,
            promised_stream_id=3
        )

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

    def test_can_handle_frames_with_invalid_padding(self, frame_factory):
        """
        Frames with invalid padding cause connection teardown.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(self.example_request_headers)
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        invalid_data_frame = (
            b'\x00\x00\x05\x00\x0b\x00\x00\x00\x01\x06\x54\x65\x73\x74'
        )

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(invalid_data_frame)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1, error_code=1
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_receiving_frames_with_insufficent_size(self, frame_factory):
        """
        Frames with not enough data cause connection teardown.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        invalid_window_update_frame = (
            b'\x00\x00\x03\x08\x00\x00\x00\x00\x00\x00\x00\x02'
        )

        with pytest.raises(h2.exceptions.FrameDataMissingError):
            c.receive_data(invalid_window_update_frame)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0, error_code=h2.errors.ErrorCodes.FRAME_SIZE_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_reject_data_on_closed_streams(self, frame_factory):
        """
        When a stream is not open to the remote peer, we reject receiving data
        frames from them.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            self.example_request_headers,
            flags=['END_STREAM']
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        bad_frame = frame_factory.build_data_frame(
            data=b'some data'
        )
        c.receive_data(bad_frame.serialize())

        expected = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        ).serialize()
        assert c.data_to_send() == expected

    def test_unexpected_continuation_on_closed_stream(self, frame_factory):
        """
        CONTINUATION frames received on closed streams cause connection errors
        of type PROTOCOL_ERROR.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            self.example_request_headers,
            flags=['END_STREAM']
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        bad_frame = frame_factory.build_continuation_frame(
            header_block=b'hello'
        )

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(bad_frame.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR,
            last_stream_id=1
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_prevent_continuation_dos(self, frame_factory):
        """
        Receiving too many CONTINUATION frames in one block causes a protocol
        error.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            self.example_request_headers,
        )
        f.flags = {'END_STREAM'}
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        # Send 63 additional frames.
        for _ in range(0, 63):
            extra_frame = frame_factory.build_continuation_frame(
                header_block=b'hello'
            )
            c.receive_data(extra_frame.serialize())

        # The final continuation frame should cause a protocol error.
        extra_frame = frame_factory.build_continuation_frame(
            header_block=b'hello'
        )
        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(extra_frame.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0,
            error_code=0x1,
        )
        assert c.data_to_send() == expected_frame.serialize()

    # These settings are a bit annoyingly anonymous, but trust me, they're bad.
    @pytest.mark.parametrize(
        "settings",
        [
            {0x2: 5},
            {0x4: 2**31},
            {0x5: 5},
            {0x5: 2**24},
        ]
    )
    def test_reject_invalid_settings_values(self, frame_factory, settings):
        """
        When a SETTINGS frame is received with invalid settings values it
        causes connection teardown with the appropriate error code.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_settings_frame(settings=settings)

        with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
            c.receive_data(f.serialize())

        assert e.value.error_code == (
            h2.errors.ErrorCodes.FLOW_CONTROL_ERROR if 0x4 in settings else
            h2.errors.ErrorCodes.PROTOCOL_ERROR
        )

    def test_invalid_frame_headers_are_protocol_errors(self, frame_factory):
        """
        When invalid frame headers are received they cause ProtocolErrors to be
        raised.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )

        # Do some annoying bit twiddling here: the stream ID is currently set
        # to '1', change it to '0'. Grab the first 9 bytes (the frame header),
        # replace any instances of the byte '\x01', and then graft it onto the
        # remaining bytes.
        frame_data = f.serialize()
        frame_data = frame_data[:9].replace(b'\x01', b'\x00') + frame_data[9:]

        with pytest.raises(h2.exceptions.ProtocolError) as e:
            c.receive_data(frame_data)

        assert "Received frame with invalid header" in str(e.value)

    def test_data_before_headers(self, frame_factory):
        """
        When data frames are received before headers
        they cause ProtocolErrors to be raised.
        """
        c = h2.connection.H2Connection(config=self.client_config)
        c.initiate_connection()
        # transition stream into OPEN
        c.send_headers(1, self.example_request_headers)

        f = frame_factory.build_data_frame(b"hello")

        with pytest.raises(h2.exceptions.ProtocolError) as e:
            c.receive_data(f.serialize())

        assert "cannot receive data before headers" in str(e.value)

    def test_get_stream_reset_event_on_auto_reset(self, frame_factory):
        """
        When hyper-h2 resets a stream automatically, a StreamReset event fires.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            self.example_request_headers,
            flags=['END_STREAM']
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        bad_frame = frame_factory.build_data_frame(
            data=b'some data'
        )
        events = c.receive_data(bad_frame.serialize())

        expected = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        ).serialize()
        assert c.data_to_send() == expected

        assert len(events) == 1
        event = events[0]
        assert isinstance(event, h2.events.StreamReset)
        assert event.stream_id == 1
        assert event.error_code == h2.errors.ErrorCodes.STREAM_CLOSED
        assert not event.remote_reset

    def test_one_one_stream_reset(self, frame_factory):
        """
        When hyper-h2 resets a stream automatically, a StreamReset event fires,
        but only for the first reset: the others are silent.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            self.example_request_headers,
            flags=['END_STREAM']
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        bad_frame = frame_factory.build_data_frame(
            data=b'some data'
        )
        # Receive 5 frames.
        events = c.receive_data(bad_frame.serialize() * 5)

        expected = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        ).serialize()
        assert c.data_to_send() == expected * 5

        assert len(events) == 1
        event = events[0]
        assert isinstance(event, h2.events.StreamReset)
        assert event.stream_id == 1
        assert event.error_code == h2.errors.ErrorCodes.STREAM_CLOSED
        assert not event.remote_reset

    @pytest.mark.parametrize('value', ['', 'twelve'])
    def test_error_on_invalid_content_length(self, frame_factory, value):
        """
        When an invalid content-length is received, a ProtocolError is thrown.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.example_request_headers + [('content-length', value)]
        )
        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_invalid_header_data_protocol_error(self, frame_factory):
        """
        If an invalid header block is received, we raise a ProtocolError.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.example_request_headers
        )
        f.data = b'\x00\x00\x00\x00'

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_invalid_push_promise_data_protocol_error(self, frame_factory):
        """
        If an invalid header block is received on a PUSH_PROMISE, we raise a
        ProtocolError.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)
        c.clear_outbound_data_buffer()

        f = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=self.example_request_headers
        )
        f.data = b'\x00\x00\x00\x00'

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_cannot_receive_push_on_pushed_stream(self, frame_factory):
        """
        If a PUSH_PROMISE frame is received with the parent stream ID being a
        pushed stream, this is rejected with a PROTOCOL_ERROR.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1,
            headers=self.example_request_headers,
            end_stream=True
        )

        f1 = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=self.example_request_headers,
        )
        f2 = frame_factory.build_headers_frame(
            stream_id=2,
            headers=self.example_response_headers,
        )
        c.receive_data(f1.serialize() + f2.serialize())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_push_promise_frame(
            stream_id=2,
            promised_stream_id=4,
            headers=self.example_request_headers,
        )

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=2,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_cannot_send_push_on_pushed_stream(self, frame_factory):
        """
        If a user tries to send a PUSH_PROMISE frame with the parent stream ID
        being a pushed stream, this is rejected with a PROTOCOL_ERROR.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        f = frame_factory.build_headers_frame(
            stream_id=1, headers=self.example_request_headers
        )
        c.receive_data(f.serialize())

        c.push_stream(
            stream_id=1,
            promised_stream_id=2,
            request_headers=self.example_request_headers
        )
        c.send_headers(stream_id=2, headers=self.example_response_headers)

        with pytest.raises(h2.exceptions.ProtocolError):
            c.push_stream(
                stream_id=2,
                promised_stream_id=4,
                request_headers=self.example_request_headers
            )
