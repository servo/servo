# -*- coding: utf-8 -*-
"""
test_stream_reset
~~~~~~~~~~~~~~~~~

More complex tests that exercise stream resetting functionality to validate
that connection state is appropriately maintained.

Specifically, these tests validate that streams that have been reset accurately
keep track of connection-level state.
"""
import pytest

import h2.connection
import h2.errors
import h2.events


class TestStreamReset(object):
    """
    Tests for resetting streams.
    """
    example_request_headers = [
        (b':authority', b'example.com'),
        (b':path', b'/'),
        (b':scheme', b'https'),
        (b':method', b'GET'),
    ]
    example_response_headers = [
        (b':status', b'200'),
        (b'server', b'fake-serv/0.1.0'),
        (b'content-length', b'0')
    ]

    def test_reset_stream_keeps_header_state_correct(self, frame_factory):
        """
        A stream that has been reset still affects the header decoder.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)
        c.reset_stream(stream_id=1)
        c.send_headers(stream_id=3, headers=self.example_request_headers)
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            headers=self.example_response_headers, stream_id=1
        )
        rst_frame = frame_factory.build_rst_stream_frame(
            1, h2.errors.ErrorCodes.STREAM_CLOSED
        )
        events = c.receive_data(f.serialize())
        assert not events
        assert c.data_to_send() == rst_frame.serialize()

        # This works because the header state should be intact from the headers
        # frame that was send on stream 1, so they should decode cleanly.
        f = frame_factory.build_headers_frame(
            headers=self.example_response_headers, stream_id=3
        )
        event = c.receive_data(f.serialize())[0]

        assert isinstance(event, h2.events.ResponseReceived)
        assert event.stream_id == 3
        assert event.headers == self.example_response_headers

    @pytest.mark.parametrize('close_id,other_id', [(1, 3), (3, 1)])
    def test_reset_stream_keeps_flow_control_correct(self,
                                                     close_id,
                                                     other_id,
                                                     frame_factory):
        """
        A stream that has been reset still affects the connection flow control
        window.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)
        c.send_headers(stream_id=3, headers=self.example_request_headers)

        # Record the initial window size.
        initial_window = c.remote_flow_control_window(stream_id=other_id)

        f = frame_factory.build_headers_frame(
            headers=self.example_response_headers, stream_id=close_id
        )
        c.receive_data(f.serialize())
        c.reset_stream(stream_id=close_id)
        c.clear_outbound_data_buffer()

        f = frame_factory.build_data_frame(
            data=b'some data!',
            stream_id=close_id
        )
        events = c.receive_data(f.serialize())

        rst_frame = frame_factory.build_rst_stream_frame(
            close_id, h2.errors.ErrorCodes.STREAM_CLOSED
        )
        assert not events
        assert c.data_to_send() == rst_frame.serialize()

        new_window = c.remote_flow_control_window(stream_id=other_id)
        assert initial_window - len(b'some data!') == new_window

    @pytest.mark.parametrize('clear_streams', [True, False])
    def test_reset_stream_automatically_resets_pushed_streams(self,
                                                              frame_factory,
                                                              clear_streams):
        """
        Resetting a stream causes RST_STREAM frames to be automatically emitted
        to close any streams pushed after the reset.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)
        c.reset_stream(stream_id=1)
        c.clear_outbound_data_buffer()

        if clear_streams:
            # Call open_outbound_streams to force the connection to clean
            # closed streams.
            c.open_outbound_streams

        f = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=self.example_request_headers,
        )
        events = c.receive_data(f.serialize())
        assert not events

        f = frame_factory.build_rst_stream_frame(
            stream_id=2,
            error_code=h2.errors.ErrorCodes.REFUSED_STREAM,
        )
        assert c.data_to_send() == f.serialize()
