# -*- coding: utf-8 -*-
"""
test_header_indexing.py
~~~~~~~~~~~~~~~~~~~~~~~

This module contains tests that use HPACK header tuples that provide additional
metadata to the hpack module about how to encode the headers.
"""
import pytest

from hpack import HeaderTuple, NeverIndexedHeaderTuple

import h2.config
import h2.connection


def assert_header_blocks_actually_equal(block_a, block_b):
    """
    Asserts that two header bocks are really, truly equal, down to the types
    of their tuples. Doesn't return anything.
    """
    assert len(block_a) == len(block_b)

    for a, b in zip(block_a, block_b):
        assert a == b
        assert a.__class__ is b.__class__


class TestHeaderIndexing(object):
    """
    Test that Hyper-h2 can correctly handle never indexed header fields using
    the appropriate hpack data structures.
    """
    example_request_headers = [
        HeaderTuple(u':authority', u'example.com'),
        HeaderTuple(u':path', u'/'),
        HeaderTuple(u':scheme', u'https'),
        HeaderTuple(u':method', u'GET'),
    ]
    bytes_example_request_headers = [
        HeaderTuple(b':authority', b'example.com'),
        HeaderTuple(b':path', b'/'),
        HeaderTuple(b':scheme', b'https'),
        HeaderTuple(b':method', b'GET'),
    ]

    extended_request_headers = [
        HeaderTuple(u':authority', u'example.com'),
        HeaderTuple(u':path', u'/'),
        HeaderTuple(u':scheme', u'https'),
        HeaderTuple(u':method', u'GET'),
        NeverIndexedHeaderTuple(u'authorization', u'realpassword'),
    ]
    bytes_extended_request_headers = [
        HeaderTuple(b':authority', b'example.com'),
        HeaderTuple(b':path', b'/'),
        HeaderTuple(b':scheme', b'https'),
        HeaderTuple(b':method', b'GET'),
        NeverIndexedHeaderTuple(b'authorization', b'realpassword'),
    ]

    example_response_headers = [
        HeaderTuple(u':status', u'200'),
        HeaderTuple(u'server', u'fake-serv/0.1.0')
    ]
    bytes_example_response_headers = [
        HeaderTuple(b':status', b'200'),
        HeaderTuple(b'server', b'fake-serv/0.1.0')
    ]

    extended_response_headers = [
        HeaderTuple(u':status', u'200'),
        HeaderTuple(u'server', u'fake-serv/0.1.0'),
        NeverIndexedHeaderTuple(u'secure', u'you-bet'),
    ]
    bytes_extended_response_headers = [
        HeaderTuple(b':status', b'200'),
        HeaderTuple(b'server', b'fake-serv/0.1.0'),
        NeverIndexedHeaderTuple(b'secure', b'you-bet'),
    ]

    server_config = h2.config.H2Configuration(client_side=False)

    @pytest.mark.parametrize(
        'headers', (
            example_request_headers,
            bytes_example_request_headers,
            extended_request_headers,
            bytes_extended_request_headers,
        )
    )
    def test_sending_header_tuples(self, headers, frame_factory):
        """
        Providing HeaderTuple and HeaderTuple subclasses preserves the metadata
        about indexing.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        # Clear the data, then send headers.
        c.clear_outbound_data_buffer()
        c.send_headers(1, headers)

        f = frame_factory.build_headers_frame(headers=headers)
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize(
        'headers', (
            example_request_headers,
            bytes_example_request_headers,
            extended_request_headers,
            bytes_extended_request_headers,
        )
    )
    def test_header_tuples_in_pushes(self, headers, frame_factory):
        """
        Providing HeaderTuple and HeaderTuple subclasses to push promises
        preserves metadata about indexing.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())

        # We can use normal headers for the request.
        f = frame_factory.build_headers_frame(
            self.example_request_headers
        )
        c.receive_data(f.serialize())

        frame_factory.refresh_encoder()
        expected_frame = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=headers,
            flags=['END_HEADERS'],
        )

        c.clear_outbound_data_buffer()
        c.push_stream(
            stream_id=1,
            promised_stream_id=2,
            request_headers=headers
        )

        assert c.data_to_send() == expected_frame.serialize()

    @pytest.mark.parametrize(
        'headers,encoding', (
            (example_request_headers, 'utf-8'),
            (bytes_example_request_headers, None),
            (extended_request_headers, 'utf-8'),
            (bytes_extended_request_headers, None),
        )
    )
    def test_header_tuples_are_decoded_request(self,
                                               headers,
                                               encoding,
                                               frame_factory):
        """
        The indexing status of the header is preserved when emitting
        RequestReceived events.
        """
        config = h2.config.H2Configuration(
            client_side=False, header_encoding=encoding
        )
        c = h2.connection.H2Connection(config=config)
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(headers)
        data = f.serialize()
        events = c.receive_data(data)

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.RequestReceived)
        assert_header_blocks_actually_equal(headers, event.headers)

    @pytest.mark.parametrize(
        'headers,encoding', (
            (example_response_headers, 'utf-8'),
            (bytes_example_response_headers, None),
            (extended_response_headers, 'utf-8'),
            (bytes_extended_response_headers, None),
        )
    )
    def test_header_tuples_are_decoded_response(self,
                                                headers,
                                                encoding,
                                                frame_factory):
        """
        The indexing status of the header is preserved when emitting
        ResponseReceived events.
        """
        config = h2.config.H2Configuration(
            header_encoding=encoding
        )
        c = h2.connection.H2Connection(config=config)
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)

        f = frame_factory.build_headers_frame(headers)
        data = f.serialize()
        events = c.receive_data(data)

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.ResponseReceived)
        assert_header_blocks_actually_equal(headers, event.headers)

    @pytest.mark.parametrize(
        'headers,encoding', (
            (example_response_headers, 'utf-8'),
            (bytes_example_response_headers, None),
            (extended_response_headers, 'utf-8'),
            (bytes_extended_response_headers, None),
        )
    )
    def test_header_tuples_are_decoded_info_response(self,
                                                     headers,
                                                     encoding,
                                                     frame_factory):
        """
        The indexing status of the header is preserved when emitting
        InformationalResponseReceived events.
        """
        # Manipulate the headers to send 100 Continue. We need to copy the list
        # to avoid breaking the example headers.
        headers = headers[:]
        if encoding:
            headers[0] = HeaderTuple(u':status', u'100')
        else:
            headers[0] = HeaderTuple(b':status', b'100')

        config = h2.config.H2Configuration(
            header_encoding=encoding
        )
        c = h2.connection.H2Connection(config=config)
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)

        f = frame_factory.build_headers_frame(headers)
        data = f.serialize()
        events = c.receive_data(data)

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.InformationalResponseReceived)
        assert_header_blocks_actually_equal(headers, event.headers)

    @pytest.mark.parametrize(
        'headers,encoding', (
            (example_response_headers, 'utf-8'),
            (bytes_example_response_headers, None),
            (extended_response_headers, 'utf-8'),
            (bytes_extended_response_headers, None),
        )
    )
    def test_header_tuples_are_decoded_trailers(self,
                                                headers,
                                                encoding,
                                                frame_factory):
        """
        The indexing status of the header is preserved when emitting
        TrailersReceived events.
        """
        # Manipulate the headers to remove the status, which shouldn't be in
        # the trailers. We need to copy the list to avoid breaking the example
        # headers.
        headers = headers[1:]

        config = h2.config.H2Configuration(
            header_encoding=encoding
        )
        c = h2.connection.H2Connection(config=config)
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)
        f = frame_factory.build_headers_frame(self.example_response_headers)
        data = f.serialize()
        c.receive_data(data)

        f = frame_factory.build_headers_frame(headers, flags=['END_STREAM'])
        data = f.serialize()
        events = c.receive_data(data)

        assert len(events) == 2
        event = events[0]

        assert isinstance(event, h2.events.TrailersReceived)
        assert_header_blocks_actually_equal(headers, event.headers)

    @pytest.mark.parametrize(
        'headers,encoding', (
            (example_request_headers, 'utf-8'),
            (bytes_example_request_headers, None),
            (extended_request_headers, 'utf-8'),
            (bytes_extended_request_headers, None),
        )
    )
    def test_header_tuples_are_decoded_push_promise(self,
                                                    headers,
                                                    encoding,
                                                    frame_factory):
        """
        The indexing status of the header is preserved when emitting
        PushedStreamReceived events.
        """
        config = h2.config.H2Configuration(
            header_encoding=encoding
        )
        c = h2.connection.H2Connection(config=config)
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)

        f = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=headers,
            flags=['END_HEADERS'],
        )
        data = f.serialize()
        events = c.receive_data(data)

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.PushedStreamReceived)
        assert_header_blocks_actually_equal(headers, event.headers)


class TestSecureHeaders(object):
    """
    Certain headers should always be transformed to their never-indexed form.
    """
    example_request_headers = [
        (u':authority', u'example.com'),
        (u':path', u'/'),
        (u':scheme', u'https'),
        (u':method', u'GET'),
    ]
    bytes_example_request_headers = [
        (b':authority', b'example.com'),
        (b':path', b'/'),
        (b':scheme', b'https'),
        (b':method', b'GET'),
    ]
    possible_auth_headers = [
        (u'authorization', u'test'),
        (u'Authorization', u'test'),
        (u'authorization', u'really long test'),
        HeaderTuple(u'authorization', u'test'),
        HeaderTuple(u'Authorization', u'test'),
        HeaderTuple(u'authorization', u'really long test'),
        NeverIndexedHeaderTuple(u'authorization', u'test'),
        NeverIndexedHeaderTuple(u'Authorization', u'test'),
        NeverIndexedHeaderTuple(u'authorization', u'really long test'),
        (b'authorization', b'test'),
        (b'Authorization', b'test'),
        (b'authorization', b'really long test'),
        HeaderTuple(b'authorization', b'test'),
        HeaderTuple(b'Authorization', b'test'),
        HeaderTuple(b'authorization', b'really long test'),
        NeverIndexedHeaderTuple(b'authorization', b'test'),
        NeverIndexedHeaderTuple(b'Authorization', b'test'),
        NeverIndexedHeaderTuple(b'authorization', b'really long test'),
        (u'proxy-authorization', u'test'),
        (u'Proxy-Authorization', u'test'),
        (u'proxy-authorization', u'really long test'),
        HeaderTuple(u'proxy-authorization', u'test'),
        HeaderTuple(u'Proxy-Authorization', u'test'),
        HeaderTuple(u'proxy-authorization', u'really long test'),
        NeverIndexedHeaderTuple(u'proxy-authorization', u'test'),
        NeverIndexedHeaderTuple(u'Proxy-Authorization', u'test'),
        NeverIndexedHeaderTuple(u'proxy-authorization', u'really long test'),
        (b'proxy-authorization', b'test'),
        (b'Proxy-Authorization', b'test'),
        (b'proxy-authorization', b'really long test'),
        HeaderTuple(b'proxy-authorization', b'test'),
        HeaderTuple(b'Proxy-Authorization', b'test'),
        HeaderTuple(b'proxy-authorization', b'really long test'),
        NeverIndexedHeaderTuple(b'proxy-authorization', b'test'),
        NeverIndexedHeaderTuple(b'Proxy-Authorization', b'test'),
        NeverIndexedHeaderTuple(b'proxy-authorization', b'really long test'),
    ]
    secured_cookie_headers = [
        (u'cookie', u'short'),
        (u'Cookie', u'short'),
        (u'cookie', u'nineteen byte cooki'),
        HeaderTuple(u'cookie', u'short'),
        HeaderTuple(u'Cookie', u'short'),
        HeaderTuple(u'cookie', u'nineteen byte cooki'),
        NeverIndexedHeaderTuple(u'cookie', u'short'),
        NeverIndexedHeaderTuple(u'Cookie', u'short'),
        NeverIndexedHeaderTuple(u'cookie', u'nineteen byte cooki'),
        NeverIndexedHeaderTuple(u'cookie', u'longer manually secured cookie'),
        (b'cookie', b'short'),
        (b'Cookie', b'short'),
        (b'cookie', b'nineteen byte cooki'),
        HeaderTuple(b'cookie', b'short'),
        HeaderTuple(b'Cookie', b'short'),
        HeaderTuple(b'cookie', b'nineteen byte cooki'),
        NeverIndexedHeaderTuple(b'cookie', b'short'),
        NeverIndexedHeaderTuple(b'Cookie', b'short'),
        NeverIndexedHeaderTuple(b'cookie', b'nineteen byte cooki'),
        NeverIndexedHeaderTuple(b'cookie', b'longer manually secured cookie'),
    ]
    unsecured_cookie_headers = [
        (u'cookie', u'twenty byte cookie!!'),
        (u'Cookie', u'twenty byte cookie!!'),
        (u'cookie', u'substantially longer than 20 byte cookie'),
        HeaderTuple(u'cookie', u'twenty byte cookie!!'),
        HeaderTuple(u'cookie', u'twenty byte cookie!!'),
        HeaderTuple(u'Cookie', u'twenty byte cookie!!'),
        (b'cookie', b'twenty byte cookie!!'),
        (b'Cookie', b'twenty byte cookie!!'),
        (b'cookie', b'substantially longer than 20 byte cookie'),
        HeaderTuple(b'cookie', b'twenty byte cookie!!'),
        HeaderTuple(b'cookie', b'twenty byte cookie!!'),
        HeaderTuple(b'Cookie', b'twenty byte cookie!!'),
    ]

    server_config = h2.config.H2Configuration(client_side=False)

    @pytest.mark.parametrize(
        'headers', (example_request_headers, bytes_example_request_headers)
    )
    @pytest.mark.parametrize('auth_header', possible_auth_headers)
    def test_authorization_headers_never_indexed(self,
                                                 headers,
                                                 auth_header,
                                                 frame_factory):
        """
        Authorization and Proxy-Authorization headers are always forced to be
        never-indexed, regardless of their form.
        """
        # Regardless of what we send, we expect it to be never indexed.
        send_headers = headers + [auth_header]
        expected_headers = headers + [
            NeverIndexedHeaderTuple(auth_header[0].lower(), auth_header[1])
        ]

        c = h2.connection.H2Connection()
        c.initiate_connection()

        # Clear the data, then send headers.
        c.clear_outbound_data_buffer()
        c.send_headers(1, send_headers)

        f = frame_factory.build_headers_frame(headers=expected_headers)
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize(
        'headers', (example_request_headers, bytes_example_request_headers)
    )
    @pytest.mark.parametrize('auth_header', possible_auth_headers)
    def test_authorization_headers_never_indexed_push(self,
                                                      headers,
                                                      auth_header,
                                                      frame_factory):
        """
        Authorization and Proxy-Authorization headers are always forced to be
        never-indexed, regardless of their form, when pushed by a server.
        """
        # Regardless of what we send, we expect it to be never indexed.
        send_headers = headers + [auth_header]
        expected_headers = headers + [
            NeverIndexedHeaderTuple(auth_header[0].lower(), auth_header[1])
        ]

        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())

        # We can use normal headers for the request.
        f = frame_factory.build_headers_frame(
            self.example_request_headers
        )
        c.receive_data(f.serialize())

        frame_factory.refresh_encoder()
        expected_frame = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=expected_headers,
            flags=['END_HEADERS'],
        )

        c.clear_outbound_data_buffer()
        c.push_stream(
            stream_id=1,
            promised_stream_id=2,
            request_headers=send_headers
        )

        assert c.data_to_send() == expected_frame.serialize()

    @pytest.mark.parametrize(
        'headers', (example_request_headers, bytes_example_request_headers)
    )
    @pytest.mark.parametrize('cookie_header', secured_cookie_headers)
    def test_short_cookie_headers_never_indexed(self,
                                                headers,
                                                cookie_header,
                                                frame_factory):
        """
        Short cookie headers, and cookies provided as NeverIndexedHeaderTuple,
        are never indexed.
        """
        # Regardless of what we send, we expect it to be never indexed.
        send_headers = headers + [cookie_header]
        expected_headers = headers + [
            NeverIndexedHeaderTuple(cookie_header[0].lower(), cookie_header[1])
        ]

        c = h2.connection.H2Connection()
        c.initiate_connection()

        # Clear the data, then send headers.
        c.clear_outbound_data_buffer()
        c.send_headers(1, send_headers)

        f = frame_factory.build_headers_frame(headers=expected_headers)
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize(
        'headers', (example_request_headers, bytes_example_request_headers)
    )
    @pytest.mark.parametrize('cookie_header', secured_cookie_headers)
    def test_short_cookie_headers_never_indexed_push(self,
                                                     headers,
                                                     cookie_header,
                                                     frame_factory):
        """
        Short cookie headers, and cookies provided as NeverIndexedHeaderTuple,
        are never indexed when pushed by servers.
        """
        # Regardless of what we send, we expect it to be never indexed.
        send_headers = headers + [cookie_header]
        expected_headers = headers + [
            NeverIndexedHeaderTuple(cookie_header[0].lower(), cookie_header[1])
        ]

        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())

        # We can use normal headers for the request.
        f = frame_factory.build_headers_frame(
            self.example_request_headers
        )
        c.receive_data(f.serialize())

        frame_factory.refresh_encoder()
        expected_frame = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=expected_headers,
            flags=['END_HEADERS'],
        )

        c.clear_outbound_data_buffer()
        c.push_stream(
            stream_id=1,
            promised_stream_id=2,
            request_headers=send_headers
        )

        assert c.data_to_send() == expected_frame.serialize()

    @pytest.mark.parametrize(
        'headers', (example_request_headers, bytes_example_request_headers)
    )
    @pytest.mark.parametrize('cookie_header', unsecured_cookie_headers)
    def test_long_cookie_headers_can_be_indexed(self,
                                                headers,
                                                cookie_header,
                                                frame_factory):
        """
        Longer cookie headers can be indexed.
        """
        # Regardless of what we send, we expect it to be indexed.
        send_headers = headers + [cookie_header]
        expected_headers = headers + [
            HeaderTuple(cookie_header[0].lower(), cookie_header[1])
        ]

        c = h2.connection.H2Connection()
        c.initiate_connection()

        # Clear the data, then send headers.
        c.clear_outbound_data_buffer()
        c.send_headers(1, send_headers)

        f = frame_factory.build_headers_frame(headers=expected_headers)
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize(
        'headers', (example_request_headers, bytes_example_request_headers)
    )
    @pytest.mark.parametrize('cookie_header', unsecured_cookie_headers)
    def test_long_cookie_headers_can_be_indexed_push(self,
                                                     headers,
                                                     cookie_header,
                                                     frame_factory):
        """
        Longer cookie headers can be indexed.
        """
        # Regardless of what we send, we expect it to be never indexed.
        send_headers = headers + [cookie_header]
        expected_headers = headers + [
            HeaderTuple(cookie_header[0].lower(), cookie_header[1])
        ]

        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())

        # We can use normal headers for the request.
        f = frame_factory.build_headers_frame(
            self.example_request_headers
        )
        c.receive_data(f.serialize())

        frame_factory.refresh_encoder()
        expected_frame = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=expected_headers,
            flags=['END_HEADERS'],
        )

        c.clear_outbound_data_buffer()
        c.push_stream(
            stream_id=1,
            promised_stream_id=2,
            request_headers=send_headers
        )

        assert c.data_to_send() == expected_frame.serialize()
