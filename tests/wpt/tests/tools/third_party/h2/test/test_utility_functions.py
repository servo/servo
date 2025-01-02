# -*- coding: utf-8 -*-
"""
test_utility_functions
~~~~~~~~~~~~~~~~~~~~~~

Tests for the various utility functions provided by hyper-h2.
"""
import pytest

import h2.config
import h2.connection
import h2.errors
import h2.events
import h2.exceptions
from h2.utilities import SizeLimitDict, extract_method_header


class TestGetNextAvailableStreamID(object):
    """
    Tests for the ``H2Connection.get_next_available_stream_id`` method.
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

    def test_returns_correct_sequence_for_clients(self, frame_factory):
        """
        For a client connection, the correct sequence of stream IDs is
        returned.
        """
        # Running the exhaustive version of this test (all 1 billion available
        # stream IDs) is too painful. For that reason, we validate that the
        # original sequence is right for the first few thousand, and then just
        # check that it terminates properly.
        #
        # Make sure that the streams get cleaned up: 8k streams floating
        # around would make this test memory-hard, and it's not supposed to be
        # a test of how much RAM your machine has.
        c = h2.connection.H2Connection()
        c.initiate_connection()
        initial_sequence = range(1, 2**13, 2)

        for expected_stream_id in initial_sequence:
            stream_id = c.get_next_available_stream_id()
            assert stream_id == expected_stream_id

            c.send_headers(
                stream_id=stream_id,
                headers=self.example_request_headers,
                end_stream=True
            )
            f = frame_factory.build_headers_frame(
                headers=self.example_response_headers,
                stream_id=stream_id,
                flags=['END_STREAM'],
            )
            c.receive_data(f.serialize())
            c.clear_outbound_data_buffer()

        # Jump up to the last available stream ID. Don't clean up the stream
        # here because who cares about one stream.
        last_client_id = 2**31 - 1
        c.send_headers(
            stream_id=last_client_id,
            headers=self.example_request_headers,
            end_stream=True
        )

        with pytest.raises(h2.exceptions.NoAvailableStreamIDError):
            c.get_next_available_stream_id()

    def test_returns_correct_sequence_for_servers(self, frame_factory):
        """
        For a server connection, the correct sequence of stream IDs is
        returned.
        """
        # Running the exhaustive version of this test (all 1 billion available
        # stream IDs) is too painful. For that reason, we validate that the
        # original sequence is right for the first few thousand, and then just
        # check that it terminates properly.
        #
        # Make sure that the streams get cleaned up: 8k streams floating
        # around would make this test memory-hard, and it's not supposed to be
        # a test of how much RAM your machine has.
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )
        c.receive_data(f.serialize())

        initial_sequence = range(2, 2**13, 2)

        for expected_stream_id in initial_sequence:
            stream_id = c.get_next_available_stream_id()
            assert stream_id == expected_stream_id

            c.push_stream(
                stream_id=1,
                promised_stream_id=stream_id,
                request_headers=self.example_request_headers
            )
            c.send_headers(
                stream_id=stream_id,
                headers=self.example_response_headers,
                end_stream=True
            )
            c.clear_outbound_data_buffer()

        # Jump up to the last available stream ID. Don't clean up the stream
        # here because who cares about one stream.
        last_server_id = 2**31 - 2
        c.push_stream(
            stream_id=1,
            promised_stream_id=last_server_id,
            request_headers=self.example_request_headers,
        )

        with pytest.raises(h2.exceptions.NoAvailableStreamIDError):
            c.get_next_available_stream_id()

    def test_does_not_increment_without_stream_send(self):
        """
        If a new stream isn't actually created, the next stream ID doesn't
        change.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        first_stream_id = c.get_next_available_stream_id()
        second_stream_id = c.get_next_available_stream_id()

        assert first_stream_id == second_stream_id

        c.send_headers(
            stream_id=first_stream_id,
            headers=self.example_request_headers
        )

        third_stream_id = c.get_next_available_stream_id()
        assert third_stream_id == (first_stream_id + 2)


class TestExtractHeader(object):

    example_request_headers = [
            (u':authority', u'example.com'),
            (u':path', u'/'),
            (u':scheme', u'https'),
            (u':method', u'GET'),
    ]
    example_headers_with_bytes = [
            (b':authority', b'example.com'),
            (b':path', b'/'),
            (b':scheme', b'https'),
            (b':method', b'GET'),
    ]

    @pytest.mark.parametrize(
        'headers', [example_request_headers, example_headers_with_bytes]
    )
    def test_extract_header_method(self, headers):
        assert extract_method_header(headers) == b'GET'


def test_size_limit_dict_limit():
    dct = SizeLimitDict(size_limit=2)

    dct[1] = 1
    dct[2] = 2

    assert len(dct) == 2
    assert dct[1] == 1
    assert dct[2] == 2

    dct[3] = 3

    assert len(dct) == 2
    assert dct[2] == 2
    assert dct[3] == 3
    assert 1 not in dct


def test_size_limit_dict_limit_init():
    initial_dct = {
        1: 1,
        2: 2,
        3: 3,
    }

    dct = SizeLimitDict(initial_dct, size_limit=2)

    assert len(dct) == 2


def test_size_limit_dict_no_limit():
    dct = SizeLimitDict(size_limit=None)

    dct[1] = 1
    dct[2] = 2

    assert len(dct) == 2
    assert dct[1] == 1
    assert dct[2] == 2

    dct[3] = 3

    assert len(dct) == 3
    assert dct[1] == 1
    assert dct[2] == 2
    assert dct[3] == 3
