# -*- coding: utf-8 -*-
"""
test_closed_streams
~~~~~~~~~~~~~~~~~~~

Tests that we handle closed streams correctly.
"""
import pytest

import h2.config
import h2.connection
import h2.errors
import h2.events
import h2.exceptions


class TestClosedStreams(object):
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

    def test_can_receive_multiple_rst_stream_frames(self, frame_factory):
        """
        Multiple RST_STREAM frames can be received, either at once or well
        after one another. Only the first fires an event.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers, end_stream=True)

        f = frame_factory.build_rst_stream_frame(stream_id=1)
        events = c.receive_data(f.serialize() * 3)

        # Force an iteration over all the streams to remove them.
        c.open_outbound_streams

        # Receive more data.
        events += c.receive_data(f.serialize() * 3)

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.StreamReset)

    def test_receiving_low_stream_id_causes_goaway(self, frame_factory):
        """
        The remote peer creating a stream with a lower ID than one we've seen
        causes a GOAWAY frame.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.initiate_connection()

        f = frame_factory.build_headers_frame(
            self.example_request_headers,
            stream_id=3,
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            self.example_request_headers,
            stream_id=1,
        )

        with pytest.raises(h2.exceptions.StreamIDTooLowError) as e:
            c.receive_data(f.serialize())

        assert e.value.stream_id == 1
        assert e.value.max_stream_id == 3

        f = frame_factory.build_goaway_frame(
            last_stream_id=3,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR,
        )
        assert c.data_to_send() == f.serialize()

    def test_closed_stream_not_present_in_streams_dict(self, frame_factory):
        """
        When streams have been closed, they get removed from the streams
        dictionary the next time we count the open streams.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.initiate_connection()

        f = frame_factory.build_headers_frame(self.example_request_headers)
        c.receive_data(f.serialize())
        c.push_stream(1, 2, self.example_request_headers)
        c.reset_stream(1)
        c.clear_outbound_data_buffer()

        f = frame_factory.build_rst_stream_frame(stream_id=2)
        c.receive_data(f.serialize())

        # Force a count of the streams.
        assert not c.open_outbound_streams

        # The streams dictionary should be empty.
        assert not c.streams

    def test_receive_rst_stream_on_closed_stream(self, frame_factory):
        """
        RST_STREAM frame should be ignored if stream is in a closed state.
        See RFC 7540 Section 5.1 (closed state)
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        # Client sends request
        c.send_headers(1, self.example_request_headers)

        # Some time passes and client sends DATA frame and closes stream,
        # so it is in a half-closed state
        c.send_data(1, b'some data', end_stream=True)

        # Server received HEADERS frame but DATA frame is still on the way.
        # Stream is in open state on the server-side. In this state server is
        # allowed to end stream and reset it - this trick helps immediately
        # close stream on the server-side.
        headers_frame = frame_factory.build_headers_frame(
            [(':status', '200')],
            flags=['END_STREAM'],
            stream_id=1,
        )
        events = c.receive_data(headers_frame.serialize())
        assert len(events) == 2
        response_received, stream_ended = events
        assert isinstance(response_received, h2.events.ResponseReceived)
        assert isinstance(stream_ended, h2.events.StreamEnded)

        rst_stream_frame = frame_factory.build_rst_stream_frame(stream_id=1)
        events = c.receive_data(rst_stream_frame.serialize())
        assert not events

    def test_receive_window_update_on_closed_stream(self, frame_factory):
        """
        WINDOW_UPDATE frame should be ignored if stream is in a closed state.
        See RFC 7540 Section 5.1 (closed state)
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        # Client sends request
        c.send_headers(1, self.example_request_headers)

        # Some time passes and client sends DATA frame and closes stream,
        # so it is in a half-closed state
        c.send_data(1, b'some data', end_stream=True)

        # Server received HEADERS frame but DATA frame is still on the way.
        # Stream is in open state on the server-side. In this state server is
        # allowed to end stream and after that acknowledge received data by
        # sending WINDOW_UPDATE frames.
        headers_frame = frame_factory.build_headers_frame(
            [(':status', '200')],
            flags=['END_STREAM'],
            stream_id=1,
        )
        events = c.receive_data(headers_frame.serialize())
        assert len(events) == 2
        response_received, stream_ended = events
        assert isinstance(response_received, h2.events.ResponseReceived)
        assert isinstance(stream_ended, h2.events.StreamEnded)

        window_update_frame = frame_factory.build_window_update_frame(
            stream_id=1,
            increment=1,
        )
        events = c.receive_data(window_update_frame.serialize())
        assert not events


class TestStreamsClosedByEndStream(object):
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

    @pytest.mark.parametrize(
        "frame",
        [
            lambda self, ff: ff.build_headers_frame(
                self.example_request_headers, flags=['END_STREAM']),
            lambda self, ff: ff.build_headers_frame(
                self.example_request_headers),
        ]
    )
    @pytest.mark.parametrize("clear_streams", [True, False])
    def test_frames_after_recv_end_will_error(self,
                                              frame_factory,
                                              frame,
                                              clear_streams):
        """
        A stream that is closed by receiving END_STREAM raises
        ProtocolError when it receives an unexpected frame.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.initiate_connection()

        f = frame_factory.build_headers_frame(
            self.example_request_headers, flags=['END_STREAM']
        )
        c.receive_data(f.serialize())
        c.send_headers(
            stream_id=1,
            headers=self.example_response_headers,
            end_stream=True
        )

        if clear_streams:
            # Call open_inbound_streams to force the connection to clean
            # closed streams.
            c.open_inbound_streams

        c.clear_outbound_data_buffer()

        f = frame(self, frame_factory)
        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

        f = frame_factory.build_goaway_frame(
            last_stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        )
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize(
        "frame",
        [
            lambda self, ff: ff.build_headers_frame(
                self.example_response_headers, flags=['END_STREAM']),
            lambda self, ff: ff.build_headers_frame(
                self.example_response_headers),
        ]
    )
    @pytest.mark.parametrize("clear_streams", [True, False])
    def test_frames_after_send_end_will_error(self,
                                              frame_factory,
                                              frame,
                                              clear_streams):
        """
        A stream that is closed by sending END_STREAM raises
        ProtocolError when it receives an unexpected frame.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers,
                       end_stream=True)

        f = frame_factory.build_headers_frame(
            self.example_response_headers, flags=['END_STREAM']
        )
        c.receive_data(f.serialize())

        if clear_streams:
            # Call open_outbound_streams to force the connection to clean
            # closed streams.
            c.open_outbound_streams

        c.clear_outbound_data_buffer()

        f = frame(self, frame_factory)
        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

        f = frame_factory.build_goaway_frame(
            last_stream_id=0,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        )
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize(
        "frame",
        [
            lambda self, ff: ff.build_window_update_frame(1, 1),
            lambda self, ff: ff.build_rst_stream_frame(1)
        ]
    )
    def test_frames_after_send_end_will_be_ignored(self,
                                                   frame_factory,
                                                   frame):
        """
        A stream that is closed by sending END_STREAM will raise
        ProtocolError when received unexpected frame.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.initiate_connection()

        f = frame_factory.build_headers_frame(
            self.example_request_headers, flags=['END_STREAM']
        )
        c.receive_data(f.serialize())
        c.send_headers(
            stream_id=1,
            headers=self.example_response_headers,
            end_stream=True
        )

        c.clear_outbound_data_buffer()

        f = frame(self, frame_factory)
        events = c.receive_data(f.serialize())

        assert not events


class TestStreamsClosedByRstStream(object):
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

    @pytest.mark.parametrize(
        "frame",
        [
            lambda self, ff: ff.build_headers_frame(
                self.example_request_headers),
            lambda self, ff: ff.build_headers_frame(
                self.example_request_headers, flags=['END_STREAM']),
        ]
    )
    def test_resets_further_frames_after_recv_reset(self,
                                                    frame_factory,
                                                    frame):
        """
        A stream that is closed by receive RST_STREAM can receive further
        frames: it simply sends RST_STREAM for it, and additionally
        WINDOW_UPDATE for DATA frames.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.initiate_connection()

        header_frame = frame_factory.build_headers_frame(
            self.example_request_headers, flags=['END_STREAM']
        )
        c.receive_data(header_frame.serialize())

        c.send_headers(
            stream_id=1,
            headers=self.example_response_headers,
            end_stream=False
        )

        rst_frame = frame_factory.build_rst_stream_frame(
            1, h2.errors.ErrorCodes.STREAM_CLOSED
        )
        c.receive_data(rst_frame.serialize())
        c.clear_outbound_data_buffer()

        f = frame(self, frame_factory)
        events = c.receive_data(f.serialize())

        rst_frame = frame_factory.build_rst_stream_frame(
            1, h2.errors.ErrorCodes.STREAM_CLOSED
        )
        assert not events
        assert c.data_to_send() == rst_frame.serialize()

        events = c.receive_data(f.serialize() * 3)
        assert not events
        assert c.data_to_send() == rst_frame.serialize() * 3

        # Iterate over the streams to make sure it's gone, then confirm the
        # behaviour is unchanged.
        c.open_outbound_streams

        events = c.receive_data(f.serialize() * 3)
        assert not events
        assert c.data_to_send() == rst_frame.serialize() * 3

    def test_resets_further_data_frames_after_recv_reset(self,
                                                         frame_factory):
        """
        A stream that is closed by receive RST_STREAM can receive further
        DATA frames: it simply sends WINDOW_UPDATE for the connection flow
        window, and RST_STREAM for the stream.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.initiate_connection()

        header_frame = frame_factory.build_headers_frame(
            self.example_request_headers, flags=['END_STREAM']
        )
        c.receive_data(header_frame.serialize())

        c.send_headers(
            stream_id=1,
            headers=self.example_response_headers,
            end_stream=False
        )

        rst_frame = frame_factory.build_rst_stream_frame(
            1, h2.errors.ErrorCodes.STREAM_CLOSED
        )
        c.receive_data(rst_frame.serialize())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_data_frame(
            data=b'some data'
        )

        events = c.receive_data(f.serialize())
        assert not events

        expected = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        ).serialize()
        assert c.data_to_send() == expected

        events = c.receive_data(f.serialize() * 3)
        assert not events
        assert c.data_to_send() == expected * 3

        # Iterate over the streams to make sure it's gone, then confirm the
        # behaviour is unchanged.
        c.open_outbound_streams

        events = c.receive_data(f.serialize() * 3)
        assert not events
        assert c.data_to_send() == expected * 3

    @pytest.mark.parametrize(
        "frame",
        [
            lambda self, ff: ff.build_headers_frame(
                self.example_request_headers),
            lambda self, ff: ff.build_headers_frame(
                self.example_request_headers, flags=['END_STREAM']),
        ]
    )
    def test_resets_further_frames_after_send_reset(self,
                                                    frame_factory,
                                                    frame):
        """
        A stream that is closed by sent RST_STREAM can receive further frames:
        it simply sends RST_STREAM for it.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.initiate_connection()

        header_frame = frame_factory.build_headers_frame(
            self.example_request_headers, flags=['END_STREAM']
        )
        c.receive_data(header_frame.serialize())

        c.send_headers(
            stream_id=1,
            headers=self.example_response_headers,
            end_stream=False
        )

        c.reset_stream(1, h2.errors.ErrorCodes.INTERNAL_ERROR)

        rst_frame = frame_factory.build_rst_stream_frame(
            1, h2.errors.ErrorCodes.STREAM_CLOSED
        )
        c.clear_outbound_data_buffer()

        f = frame(self, frame_factory)
        events = c.receive_data(f.serialize())

        rst_frame = frame_factory.build_rst_stream_frame(
            1, h2.errors.ErrorCodes.STREAM_CLOSED
        )
        assert not events
        assert c.data_to_send() == rst_frame.serialize()

        events = c.receive_data(f.serialize() * 3)
        assert not events
        assert c.data_to_send() == rst_frame.serialize() * 3

        # Iterate over the streams to make sure it's gone, then confirm the
        # behaviour is unchanged.
        c.open_outbound_streams

        events = c.receive_data(f.serialize() * 3)
        assert not events
        assert c.data_to_send() == rst_frame.serialize() * 3

    def test_resets_further_data_frames_after_send_reset(self,
                                                         frame_factory):
        """
        A stream that is closed by sent RST_STREAM can receive further
        data frames: it simply sends WINDOW_UPDATE and RST_STREAM for it.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.initiate_connection()

        header_frame = frame_factory.build_headers_frame(
            self.example_request_headers, flags=['END_STREAM']
        )
        c.receive_data(header_frame.serialize())

        c.send_headers(
            stream_id=1,
            headers=self.example_response_headers,
            end_stream=False
        )

        c.reset_stream(1, h2.errors.ErrorCodes.INTERNAL_ERROR)

        c.clear_outbound_data_buffer()

        f = frame_factory.build_data_frame(
            data=b'some data'
        )
        events = c.receive_data(f.serialize())
        assert not events
        expected = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        ).serialize()
        assert c.data_to_send() == expected

        events = c.receive_data(f.serialize() * 3)
        assert not events
        assert c.data_to_send() == expected * 3

        # Iterate over the streams to make sure it's gone, then confirm the
        # behaviour is unchanged.
        c.open_outbound_streams

        events = c.receive_data(f.serialize() * 3)
        assert not events
        assert c.data_to_send() == expected * 3
