# -*- coding: utf-8 -*-
"""
test_h2_upgrade.py
~~~~~~~~~~~~~~~~~~

This module contains tests that exercise the HTTP Upgrade functionality of
hyper-h2, ensuring that clients and servers can upgrade their plaintext
HTTP/1.1 connections to HTTP/2.
"""
import base64

import pytest

import h2.config
import h2.connection
import h2.errors
import h2.events
import h2.exceptions


class TestClientUpgrade(object):
    """
    Tests of the client-side of the HTTP/2 upgrade dance.
    """
    example_request_headers = [
        (b':authority', b'example.com'),
        (b':path', b'/'),
        (b':scheme', b'https'),
        (b':method', b'GET'),
    ]
    example_response_headers = [
        (b':status', b'200'),
        (b'server', b'fake-serv/0.1.0')
    ]

    def test_returns_http2_settings(self, frame_factory):
        """
        Calling initiate_upgrade_connection returns a base64url encoded
        Settings frame with the settings used by the connection.
        """
        conn = h2.connection.H2Connection()
        data = conn.initiate_upgrade_connection()

        # The base64 encoding must not be padded.
        assert not data.endswith(b'=')

        # However, SETTINGS frames should never need to be padded.
        decoded_frame = base64.urlsafe_b64decode(data)
        expected_frame = frame_factory.build_settings_frame(
            settings=conn.local_settings
        )
        assert decoded_frame == expected_frame.serialize_body()

    def test_emits_preamble(self, frame_factory):
        """
        Calling initiate_upgrade_connection emits the connection preamble.
        """
        conn = h2.connection.H2Connection()
        conn.initiate_upgrade_connection()

        data = conn.data_to_send()
        assert data.startswith(frame_factory.preamble())

        data = data[len(frame_factory.preamble()):]
        expected_frame = frame_factory.build_settings_frame(
            settings=conn.local_settings
        )
        assert data == expected_frame.serialize()

    def test_can_receive_response(self, frame_factory):
        """
        After upgrading, we can safely receive a response.
        """
        c = h2.connection.H2Connection()
        c.initiate_upgrade_connection()
        c.clear_outbound_data_buffer()

        f1 = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.example_response_headers,
        )
        f2 = frame_factory.build_data_frame(
            stream_id=1,
            data=b'some data',
            flags=['END_STREAM']
        )
        events = c.receive_data(f1.serialize() + f2.serialize())
        assert len(events) == 3

        assert isinstance(events[0], h2.events.ResponseReceived)
        assert isinstance(events[1], h2.events.DataReceived)
        assert isinstance(events[2], h2.events.StreamEnded)

        assert events[0].headers == self.example_response_headers
        assert events[1].data == b'some data'
        assert all(e.stream_id == 1 for e in events)

        assert not c.data_to_send()

    def test_can_receive_pushed_stream(self, frame_factory):
        """
        After upgrading, we can safely receive a pushed stream.
        """
        c = h2.connection.H2Connection()
        c.initiate_upgrade_connection()
        c.clear_outbound_data_buffer()

        f = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=self.example_request_headers,
        )
        events = c.receive_data(f.serialize())
        assert len(events) == 1

        assert isinstance(events[0], h2.events.PushedStreamReceived)
        assert events[0].headers == self.example_request_headers
        assert events[0].parent_stream_id == 1
        assert events[0].pushed_stream_id == 2

    def test_cannot_send_headers_stream_1(self, frame_factory):
        """
        After upgrading, we cannot send headers on stream 1.
        """
        c = h2.connection.H2Connection()
        c.initiate_upgrade_connection()
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.send_headers(stream_id=1, headers=self.example_request_headers)

    def test_cannot_send_data_stream_1(self, frame_factory):
        """
        After upgrading, we cannot send data on stream 1.
        """
        c = h2.connection.H2Connection()
        c.initiate_upgrade_connection()
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.send_data(stream_id=1, data=b'some data')


class TestServerUpgrade(object):
    """
    Tests of the server-side of the HTTP/2 upgrade dance.
    """
    example_request_headers = [
        (b':authority', b'example.com'),
        (b':path', b'/'),
        (b':scheme', b'https'),
        (b':method', b'GET'),
    ]
    example_response_headers = [
        (b':status', b'200'),
        (b'server', b'fake-serv/0.1.0')
    ]
    server_config = h2.config.H2Configuration(client_side=False)

    def test_returns_nothing(self, frame_factory):
        """
        Calling initiate_upgrade_connection returns nothing.
        """
        conn = h2.connection.H2Connection(config=self.server_config)
        curl_header = b"AAMAAABkAAQAAP__"
        data = conn.initiate_upgrade_connection(curl_header)
        assert data is None

    def test_emits_preamble(self, frame_factory):
        """
        Calling initiate_upgrade_connection emits the connection preamble.
        """
        conn = h2.connection.H2Connection(config=self.server_config)
        conn.initiate_upgrade_connection()

        data = conn.data_to_send()
        expected_frame = frame_factory.build_settings_frame(
            settings=conn.local_settings
        )
        assert data == expected_frame.serialize()

    def test_can_send_response(self, frame_factory):
        """
        After upgrading, we can safely send a response.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_upgrade_connection()
        c.clear_outbound_data_buffer()

        c.send_headers(stream_id=1, headers=self.example_response_headers)
        c.send_data(stream_id=1, data=b'some data', end_stream=True)

        f1 = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.example_response_headers,
        )
        f2 = frame_factory.build_data_frame(
            stream_id=1,
            data=b'some data',
            flags=['END_STREAM']
        )

        expected_data = f1.serialize() + f2.serialize()
        assert c.data_to_send() == expected_data

    def test_can_push_stream(self, frame_factory):
        """
        After upgrading, we can safely push a stream.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_upgrade_connection()
        c.clear_outbound_data_buffer()

        c.push_stream(
            stream_id=1,
            promised_stream_id=2,
            request_headers=self.example_request_headers
        )

        f = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=self.example_request_headers,
        )
        assert c.data_to_send() == f.serialize()

    def test_cannot_receive_headers_stream_1(self, frame_factory):
        """
        After upgrading, we cannot receive headers on stream 1.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_upgrade_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.example_request_headers,
        )
        c.receive_data(f.serialize())

        expected_frame = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_cannot_receive_data_stream_1(self, frame_factory):
        """
        After upgrading, we cannot receive data on stream 1.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_upgrade_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_data_frame(
            stream_id=1,
            data=b'some data',
        )
        c.receive_data(f.serialize())

        expected_frame = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_client_settings_are_applied(self, frame_factory):
        """
        The settings provided by the client are applied and immediately
        ACK'ed.
        """
        server = h2.connection.H2Connection(config=self.server_config)
        client = h2.connection.H2Connection()

        # As a precaution, let's confirm that the server and client, at the
        # start of the connection, do not agree on their initial settings
        # state.
        assert (
            client.local_settings._settings != server.remote_settings._settings
        )

        # Get the client header data and pass it to the server.
        header_data = client.initiate_upgrade_connection()
        server.initiate_upgrade_connection(header_data)

        # This gets complex, but here we go.
        # RFC 7540 § 3.2.1 says that "explicit acknowledgement" of the settings
        # in the header is "not necessary". That's annoyingly vague, but we
        # interpret that to mean "should not be sent". So to test that this
        # worked we need to test that the server has only sent the preamble,
        # and has not sent a SETTINGS ack, and also that the server has the
        # correct settings.
        expected_frame = frame_factory.build_settings_frame(
            server.local_settings
        )
        assert server.data_to_send() == expected_frame.serialize()

        # We violate abstraction layers here, but I don't think defining __eq__
        # for this is worth it. In this case, both the client and server should
        # agree that these settings have been ACK'd, so their underlying
        # dictionaries should be identical.
        assert (
            client.local_settings._settings == server.remote_settings._settings
        )
