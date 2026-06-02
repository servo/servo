# -*- coding: utf-8 -*-
"""
test_invalid_content_lengths.py
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This module contains tests that use invalid content lengths, and validates that
they fail appropriately.
"""
import pytest

import h2.config
import h2.connection
import h2.errors
import h2.events
import h2.exceptions


class TestInvalidContentLengths(object):
    """
    Hyper-h2 raises Protocol Errors when the content-length sent by a remote
    peer is not valid.
    """
    example_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'POST'),
        ('content-length', '15'),
    ]
    example_response_headers = [
        (':status', '200'),
        ('server', 'fake-serv/0.1.0')
    ]
    server_config = h2.config.H2Configuration(client_side=False)

    def test_too_much_data(self, frame_factory):
        """
        Remote peers sending data in excess of content-length causes Protocol
        Errors.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        headers = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )
        first_data = frame_factory.build_data_frame(data=b'\x01'*15)
        c.receive_data(headers.serialize() + first_data.serialize())
        c.clear_outbound_data_buffer()

        second_data = frame_factory.build_data_frame(data=b'\x01')
        with pytest.raises(h2.exceptions.InvalidBodyLengthError) as exp:
            c.receive_data(second_data.serialize())

        assert exp.value.expected_length == 15
        assert exp.value.actual_length == 16
        assert str(exp.value) == (
            "InvalidBodyLengthError: Expected 15 bytes, received 16"
        )

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR,
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_insufficient_data(self, frame_factory):
        """
        Remote peers sending less data than content-length causes Protocol
        Errors.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        headers = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )
        first_data = frame_factory.build_data_frame(data=b'\x01'*13)
        c.receive_data(headers.serialize() + first_data.serialize())
        c.clear_outbound_data_buffer()

        second_data = frame_factory.build_data_frame(
            data=b'\x01',
            flags=['END_STREAM'],
        )
        with pytest.raises(h2.exceptions.InvalidBodyLengthError) as exp:
            c.receive_data(second_data.serialize())

        assert exp.value.expected_length == 15
        assert exp.value.actual_length == 14
        assert str(exp.value) == (
            "InvalidBodyLengthError: Expected 15 bytes, received 14"
        )

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR,
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_insufficient_data_empty_frame(self, frame_factory):
        """
        Remote peers sending less data than content-length where the last data
        frame is empty causes Protocol Errors.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        headers = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )
        first_data = frame_factory.build_data_frame(data=b'\x01'*14)
        c.receive_data(headers.serialize() + first_data.serialize())
        c.clear_outbound_data_buffer()

        second_data = frame_factory.build_data_frame(
            data=b'',
            flags=['END_STREAM'],
        )
        with pytest.raises(h2.exceptions.InvalidBodyLengthError) as exp:
            c.receive_data(second_data.serialize())

        assert exp.value.expected_length == 15
        assert exp.value.actual_length == 14
        assert str(exp.value) == (
            "InvalidBodyLengthError: Expected 15 bytes, received 14"
        )

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR,
        )
        assert c.data_to_send() == expected_frame.serialize()
