# -*- coding: utf-8 -*-
"""
test_flow_control
~~~~~~~~~~~~~~~~~

Tests of the flow control management in h2
"""
import pytest

from hypothesis import given, settings, HealthCheck
from hypothesis.strategies import integers

import h2.config
import h2.connection
import h2.errors
import h2.events
import h2.exceptions
import h2.settings


class TestFlowControl(object):
    """
    Tests of the flow control management in the connection objects.
    """
    example_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'GET'),
    ]
    server_config = h2.config.H2Configuration(client_side=False)

    DEFAULT_FLOW_WINDOW = 65535

    def test_flow_control_initializes_properly(self):
        """
        The flow control window for a stream should initially be the default
        flow control value.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)

        assert c.local_flow_control_window(1) == self.DEFAULT_FLOW_WINDOW
        assert c.remote_flow_control_window(1) == self.DEFAULT_FLOW_WINDOW

    def test_flow_control_decreases_with_sent_data(self):
        """
        When data is sent on a stream, the flow control window should drop.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)
        c.send_data(1, b'some data')

        remaining_length = self.DEFAULT_FLOW_WINDOW - len(b'some data')
        assert (c.local_flow_control_window(1) == remaining_length)

    @pytest.mark.parametrize("pad_length", [5, 0])
    def test_flow_control_decreases_with_sent_data_with_padding(self,
                                                                pad_length):
        """
        When padded data is sent on a stream, the flow control window drops
        by the length of the padding plus 1 for the 1-byte padding length
        field.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)

        c.send_data(1, b'some data', pad_length=pad_length)
        remaining_length = (
            self.DEFAULT_FLOW_WINDOW - len(b'some data') - pad_length - 1
        )
        assert c.local_flow_control_window(1) == remaining_length

    def test_flow_control_decreases_with_received_data(self, frame_factory):
        """
        When data is received on a stream, the remote flow control window
        should drop.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        f1 = frame_factory.build_headers_frame(self.example_request_headers)
        f2 = frame_factory.build_data_frame(b'some data')

        c.receive_data(f1.serialize() + f2.serialize())

        remaining_length = self.DEFAULT_FLOW_WINDOW - len(b'some data')
        assert (c.remote_flow_control_window(1) == remaining_length)

    def test_flow_control_decreases_with_padded_data(self, frame_factory):
        """
        When padded data is received on a stream, the remote flow control
        window drops by an amount that includes the padding.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        f1 = frame_factory.build_headers_frame(self.example_request_headers)
        f2 = frame_factory.build_data_frame(b'some data', padding_len=10)

        c.receive_data(f1.serialize() + f2.serialize())

        remaining_length = (
            self.DEFAULT_FLOW_WINDOW - len(b'some data') - 10 - 1
        )
        assert (c.remote_flow_control_window(1) == remaining_length)

    def test_flow_control_is_limited_by_connection(self):
        """
        The flow control window is limited by the flow control of the
        connection.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)
        c.send_data(1, b'some data')
        c.send_headers(3, self.example_request_headers)

        remaining_length = self.DEFAULT_FLOW_WINDOW - len(b'some data')
        assert (c.local_flow_control_window(3) == remaining_length)

    def test_remote_flow_control_is_limited_by_connection(self, frame_factory):
        """
        The remote flow control window is limited by the flow control of the
        connection.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        f1 = frame_factory.build_headers_frame(self.example_request_headers)
        f2 = frame_factory.build_data_frame(b'some data')
        f3 = frame_factory.build_headers_frame(
            self.example_request_headers,
            stream_id=3,
        )
        c.receive_data(f1.serialize() + f2.serialize() + f3.serialize())

        remaining_length = self.DEFAULT_FLOW_WINDOW - len(b'some data')
        assert (c.remote_flow_control_window(3) == remaining_length)

    def test_cannot_send_more_data_than_window(self):
        """
        Sending more data than the remaining flow control window raises a
        FlowControlError.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)
        c.outbound_flow_control_window = 5

        with pytest.raises(h2.exceptions.FlowControlError):
            c.send_data(1, b'some data')

    def test_increasing_connection_window_allows_sending(self, frame_factory):
        """
        Confirm that sending a WindowUpdate frame on the connection frees
        up space for further frames.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)
        c.outbound_flow_control_window = 5

        with pytest.raises(h2.exceptions.FlowControlError):
            c.send_data(1, b'some data')

        f = frame_factory.build_window_update_frame(
            stream_id=0,
            increment=5,
        )
        c.receive_data(f.serialize())

        c.clear_outbound_data_buffer()
        c.send_data(1, b'some data')
        assert c.data_to_send()

    def test_increasing_stream_window_allows_sending(self, frame_factory):
        """
        Confirm that sending a WindowUpdate frame on the connection frees
        up space for further frames.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)
        c._get_stream_by_id(1).outbound_flow_control_window = 5

        with pytest.raises(h2.exceptions.FlowControlError):
            c.send_data(1, b'some data')

        f = frame_factory.build_window_update_frame(
            stream_id=1,
            increment=5,
        )
        c.receive_data(f.serialize())

        c.clear_outbound_data_buffer()
        c.send_data(1, b'some data')
        assert c.data_to_send()

    def test_flow_control_shrinks_in_response_to_settings(self, frame_factory):
        """
        Acknowledging SETTINGS_INITIAL_WINDOW_SIZE shrinks the flow control
        window.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)

        assert c.local_flow_control_window(1) == 65535

        f = frame_factory.build_settings_frame(
            settings={h2.settings.SettingCodes.INITIAL_WINDOW_SIZE: 1280}
        )
        c.receive_data(f.serialize())

        assert c.local_flow_control_window(1) == 1280

    def test_flow_control_grows_in_response_to_settings(self, frame_factory):
        """
        Acknowledging SETTINGS_INITIAL_WINDOW_SIZE grows the flow control
        window.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)

        # Greatly increase the connection flow control window.
        f = frame_factory.build_window_update_frame(
            stream_id=0, increment=128000
        )
        c.receive_data(f.serialize())

        # The stream flow control window is the bottleneck here.
        assert c.local_flow_control_window(1) == 65535

        f = frame_factory.build_settings_frame(
            settings={h2.settings.SettingCodes.INITIAL_WINDOW_SIZE: 128000}
        )
        c.receive_data(f.serialize())

        # The stream window is still the bottleneck, but larger now.
        assert c.local_flow_control_window(1) == 128000

    def test_flow_control_settings_blocked_by_conn_window(self, frame_factory):
        """
        Changing SETTINGS_INITIAL_WINDOW_SIZE does not affect the effective
        flow control window if the connection window isn't changed.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)

        assert c.local_flow_control_window(1) == 65535

        f = frame_factory.build_settings_frame(
            settings={h2.settings.SettingCodes.INITIAL_WINDOW_SIZE: 128000}
        )
        c.receive_data(f.serialize())

        assert c.local_flow_control_window(1) == 65535

    def test_new_streams_have_flow_control_per_settings(self, frame_factory):
        """
        After a SETTINGS_INITIAL_WINDOW_SIZE change is received, new streams
        have appropriate new flow control windows.
        """
        c = h2.connection.H2Connection()

        f = frame_factory.build_settings_frame(
            settings={h2.settings.SettingCodes.INITIAL_WINDOW_SIZE: 128000}
        )
        c.receive_data(f.serialize())

        # Greatly increase the connection flow control window.
        f = frame_factory.build_window_update_frame(
            stream_id=0, increment=128000
        )
        c.receive_data(f.serialize())

        c.send_headers(1, self.example_request_headers)
        assert c.local_flow_control_window(1) == 128000

    def test_window_update_no_stream(self, frame_factory):
        """
        WindowUpdate frames received without streams fire an appropriate
        WindowUpdated event.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_window_update_frame(
            stream_id=0,
            increment=5
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.WindowUpdated)
        assert event.stream_id == 0
        assert event.delta == 5

    def test_window_update_with_stream(self, frame_factory):
        """
        WindowUpdate frames received with streams fire an appropriate
        WindowUpdated event.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())

        f1 = frame_factory.build_headers_frame(self.example_request_headers)
        f2 = frame_factory.build_window_update_frame(
            stream_id=1,
            increment=66
        )
        data = b''.join(map(lambda f: f.serialize(), [f1, f2]))
        events = c.receive_data(data)

        assert len(events) == 2
        event = events[1]

        assert isinstance(event, h2.events.WindowUpdated)
        assert event.stream_id == 1
        assert event.delta == 66

    def test_we_can_increment_stream_flow_control(self, frame_factory):
        """
        It is possible for the user to increase the flow control window for
        streams.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers, end_stream=True)
        c.clear_outbound_data_buffer()

        expected_frame = frame_factory.build_window_update_frame(
            stream_id=1,
            increment=5
        )

        events = c.increment_flow_control_window(increment=5, stream_id=1)
        assert not events
        assert c.data_to_send() == expected_frame.serialize()

    def test_we_can_increment_connection_flow_control(self, frame_factory):
        """
        It is possible for the user to increase the flow control window for
        the entire connection.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers, end_stream=True)
        c.clear_outbound_data_buffer()

        expected_frame = frame_factory.build_window_update_frame(
            stream_id=0,
            increment=5
        )

        events = c.increment_flow_control_window(increment=5)
        assert not events
        assert c.data_to_send() == expected_frame.serialize()

    def test_we_enforce_our_flow_control_window(self, frame_factory):
        """
        The user can set a low flow control window, which leads to connection
        teardown if violated.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())

        # Change the flow control window to 80 bytes.
        c.update_settings(
            {h2.settings.SettingCodes.INITIAL_WINDOW_SIZE: 80}
        )
        f = frame_factory.build_settings_frame({}, ack=True)
        c.receive_data(f.serialize())

        # Receive a new stream.
        f = frame_factory.build_headers_frame(self.example_request_headers)
        c.receive_data(f.serialize())

        # Attempt to violate the flow control window.
        c.clear_outbound_data_buffer()
        f = frame_factory.build_data_frame(b'\x01' * 100)

        with pytest.raises(h2.exceptions.FlowControlError):
            c.receive_data(f.serialize())

        # Verify we tear down appropriately.
        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1,
            error_code=h2.errors.ErrorCodes.FLOW_CONTROL_ERROR,
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_shrink_remote_flow_control_settings(self, frame_factory):
        """
        The remote peer acknowledging our SETTINGS_INITIAL_WINDOW_SIZE shrinks
        the flow control window.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)

        assert c.remote_flow_control_window(1) == 65535

        c.update_settings({h2.settings.SettingCodes.INITIAL_WINDOW_SIZE: 1280})

        f = frame_factory.build_settings_frame({}, ack=True)
        c.receive_data(f.serialize())

        assert c.remote_flow_control_window(1) == 1280

    def test_grow_remote_flow_control_settings(self, frame_factory):
        """
        The remote peer acknowledging our SETTINGS_INITIAL_WINDOW_SIZE grows
        the flow control window.
        """
        c = h2.connection.H2Connection()
        c.send_headers(1, self.example_request_headers)

        # Increase the connection flow control window greatly.
        c.increment_flow_control_window(increment=128000)

        assert c.remote_flow_control_window(1) == 65535

        c.update_settings(
            {h2.settings.SettingCodes.INITIAL_WINDOW_SIZE: 128000}
        )
        f = frame_factory.build_settings_frame({}, ack=True)
        c.receive_data(f.serialize())

        assert c.remote_flow_control_window(1) == 128000

    def test_new_streams_have_remote_flow_control(self, frame_factory):
        """
        After a SETTINGS_INITIAL_WINDOW_SIZE change is acknowledged by the
        remote peer, new streams have appropriate new flow control windows.
        """
        c = h2.connection.H2Connection()

        c.update_settings(
            {h2.settings.SettingCodes.INITIAL_WINDOW_SIZE: 128000}
        )
        f = frame_factory.build_settings_frame({}, ack=True)
        c.receive_data(f.serialize())

        # Increase the connection flow control window greatly.
        c.increment_flow_control_window(increment=128000)

        c.send_headers(1, self.example_request_headers)
        assert c.remote_flow_control_window(1) == 128000

    @pytest.mark.parametrize(
        'increment', [0, -15, 2**31]
    )
    def test_reject_bad_attempts_to_increment_flow_control(self, increment):
        """
        Attempting to increment a flow control increment outside the valid
        range causes a ValueError to be raised.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers, end_stream=True)
        c.clear_outbound_data_buffer()

        # Fails both on and off streams.
        with pytest.raises(ValueError):
            c.increment_flow_control_window(increment=increment, stream_id=1)

        with pytest.raises(ValueError):
            c.increment_flow_control_window(increment=increment)

    @pytest.mark.parametrize('stream_id', [0, 1])
    def test_reject_bad_remote_increments(self, frame_factory, stream_id):
        """
        Remote peers attempting to increment flow control outside the valid
        range cause connection errors of type PROTOCOL_ERROR.
        """
        # The only number that can be encoded in a WINDOW_UPDATE frame but
        # isn't valid is 0.
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers, end_stream=True)
        c.clear_outbound_data_buffer()

        f = frame_factory.build_window_update_frame(
            stream_id=stream_id, increment=0
        )

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR,
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_reject_increasing_connection_window_too_far(self, frame_factory):
        """
        Attempts by the remote peer to increase the connection flow control
        window beyond 2**31 - 1 are rejected.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.clear_outbound_data_buffer()

        increment = 2**31 - c.outbound_flow_control_window

        f = frame_factory.build_window_update_frame(
            stream_id=0, increment=increment
        )

        with pytest.raises(h2.exceptions.FlowControlError):
            c.receive_data(f.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0,
            error_code=h2.errors.ErrorCodes.FLOW_CONTROL_ERROR,
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_reject_increasing_stream_window_too_far(self, frame_factory):
        """
        Attempts by the remote peer to increase the stream flow control window
        beyond 2**31 - 1 are rejected.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers)
        c.clear_outbound_data_buffer()

        increment = 2**31 - c.outbound_flow_control_window

        f = frame_factory.build_window_update_frame(
            stream_id=1, increment=increment
        )

        events = c.receive_data(f.serialize())
        assert len(events) == 1

        event = events[0]
        assert isinstance(event, h2.events.StreamReset)
        assert event.stream_id == 1
        assert event.error_code == h2.errors.ErrorCodes.FLOW_CONTROL_ERROR
        assert not event.remote_reset

        expected_frame = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.FLOW_CONTROL_ERROR,
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_reject_overlarge_conn_window_settings(self, frame_factory):
        """
        SETTINGS frames cannot change the size of the connection flow control
        window.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        # Go one byte smaller than the limit.
        increment = 2**31 - 1 - c.outbound_flow_control_window

        f = frame_factory.build_window_update_frame(
            stream_id=0, increment=increment
        )
        c.receive_data(f.serialize())

        # Receive an increment to the initial window size.
        f = frame_factory.build_settings_frame(
            settings={
                h2.settings.SettingCodes.INITIAL_WINDOW_SIZE:
                    self.DEFAULT_FLOW_WINDOW + 1
            }
        )
        c.clear_outbound_data_buffer()

        # No error is encountered.
        events = c.receive_data(f.serialize())
        assert len(events) == 1
        assert isinstance(events[0], h2.events.RemoteSettingsChanged)

        expected_frame = frame_factory.build_settings_frame(
            settings={},
            ack=True
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_reject_overlarge_stream_window_settings(self, frame_factory):
        """
        Remote attempts to create overlarge stream windows via SETTINGS frames
        are rejected.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers)

        # Go one byte smaller than the limit.
        increment = 2**31 - 1 - c.outbound_flow_control_window

        f = frame_factory.build_window_update_frame(
            stream_id=1, increment=increment
        )
        c.receive_data(f.serialize())

        # Receive an increment to the initial window size.
        f = frame_factory.build_settings_frame(
            settings={
                h2.settings.SettingCodes.INITIAL_WINDOW_SIZE:
                    self.DEFAULT_FLOW_WINDOW + 1
            }
        )
        c.clear_outbound_data_buffer()
        with pytest.raises(h2.exceptions.FlowControlError):
            c.receive_data(f.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0,
            error_code=h2.errors.ErrorCodes.FLOW_CONTROL_ERROR,
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_reject_local_overlarge_increase_connection_window(self):
        """
        Local attempts to increase the connection window too far are rejected.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        increment = 2**31 - c.inbound_flow_control_window

        with pytest.raises(h2.exceptions.FlowControlError):
            c.increment_flow_control_window(increment=increment)

    def test_reject_local_overlarge_increase_stream_window(self):
        """
        Local attempts to increase the connection window too far are rejected.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers)

        increment = 2**31 - c.inbound_flow_control_window

        with pytest.raises(h2.exceptions.FlowControlError):
            c.increment_flow_control_window(increment=increment, stream_id=1)

    def test_send_update_on_closed_streams(self, frame_factory):
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(1, self.example_request_headers)
        c.reset_stream(1)

        c.clear_outbound_data_buffer()
        c.open_outbound_streams
        c.open_inbound_streams

        f = frame_factory.build_data_frame(b'some data'*1500)
        events = c.receive_data(f.serialize()*3)
        assert not events

        expected = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        ).serialize() * 2 + frame_factory.build_window_update_frame(
            stream_id=0,
            increment=40500,
        ).serialize() + frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        ).serialize()
        assert c.data_to_send() == expected

        f = frame_factory.build_data_frame(b'')
        events = c.receive_data(f.serialize())
        assert not events

        expected = frame_factory.build_rst_stream_frame(
            stream_id=1,
            error_code=h2.errors.ErrorCodes.STREAM_CLOSED,
        ).serialize()
        assert c.data_to_send() == expected


class TestAutomaticFlowControl(object):
    """
    Tests for the automatic flow control logic.
    """
    example_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'GET'),
    ]
    server_config = h2.config.H2Configuration(client_side=False)

    DEFAULT_FLOW_WINDOW = 65535

    def _setup_connection_and_send_headers(self, frame_factory):
        """
        Setup a server-side H2Connection and send a headers frame, and then
        clear the outbound data buffer. Also increase the maximum frame size.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        c.update_settings(
            {h2.settings.SettingCodes.MAX_FRAME_SIZE: self.DEFAULT_FLOW_WINDOW}
        )
        settings_frame = frame_factory.build_settings_frame(
            settings={}, ack=True
        )
        c.receive_data(settings_frame.serialize())
        c.clear_outbound_data_buffer()

        headers_frame = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )
        c.receive_data(headers_frame.serialize())
        c.clear_outbound_data_buffer()
        return c

    @given(stream_id=integers(max_value=0))
    @settings(suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_must_acknowledge_for_stream(self, frame_factory, stream_id):
        """
        Flow control acknowledgements must be done on a stream ID that is
        greater than zero.
        """
        # We need to refresh the encoder because hypothesis has a problem with
        # integrating with py.test, meaning that we use the same frame factory
        # for all tests.
        # See https://github.com/HypothesisWorks/hypothesis-python/issues/377
        frame_factory.refresh_encoder()

        # Create a connection in a state that might actually accept
        # data acknolwedgement.
        c = self._setup_connection_and_send_headers(frame_factory)
        data_frame = frame_factory.build_data_frame(
            b'some data', flags=['END_STREAM']
        )
        c.receive_data(data_frame.serialize())

        with pytest.raises(ValueError):
            c.acknowledge_received_data(
                acknowledged_size=5, stream_id=stream_id
            )

    @given(size=integers(max_value=-1))
    @settings(suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_cannot_acknowledge_less_than_zero(self, frame_factory, size):
        """
        The user must acknowledge at least 0 bytes.
        """
        # We need to refresh the encoder because hypothesis has a problem with
        # integrating with py.test, meaning that we use the same frame factory
        # for all tests.
        # See https://github.com/HypothesisWorks/hypothesis-python/issues/377
        frame_factory.refresh_encoder()

        # Create a connection in a state that might actually accept
        # data acknolwedgement.
        c = self._setup_connection_and_send_headers(frame_factory)
        data_frame = frame_factory.build_data_frame(
            b'some data', flags=['END_STREAM']
        )
        c.receive_data(data_frame.serialize())

        with pytest.raises(ValueError):
            c.acknowledge_received_data(acknowledged_size=size, stream_id=1)

    def test_acknowledging_small_chunks_does_nothing(self, frame_factory):
        """
        When a small amount of data is received and acknowledged, no window
        update is emitted.
        """
        c = self._setup_connection_and_send_headers(frame_factory)

        data_frame = frame_factory.build_data_frame(
            b'some data', flags=['END_STREAM']
        )
        data_event = c.receive_data(data_frame.serialize())[0]

        c.acknowledge_received_data(
            data_event.flow_controlled_length, stream_id=1
        )

        assert not c.data_to_send()

    def test_acknowledging_no_data_does_nothing(self, frame_factory):
        """
        If a user accidentally acknowledges no data, nothing happens.
        """
        c = self._setup_connection_and_send_headers(frame_factory)

        # Send an empty data frame, just to give the user impetus to ack the
        # data.
        data_frame = frame_factory.build_data_frame(b'')
        c.receive_data(data_frame.serialize())

        c.acknowledge_received_data(0, stream_id=1)
        assert not c.data_to_send()

    @pytest.mark.parametrize('force_cleanup', (True, False))
    def test_acknowledging_data_on_closed_stream(self,
                                                 frame_factory,
                                                 force_cleanup):
        """
        When acknowledging data on a stream that has just been closed, no
        acknowledgement is given for that stream, only for the connection.
        """
        c = self._setup_connection_and_send_headers(frame_factory)

        data_to_send = b'\x00' * self.DEFAULT_FLOW_WINDOW
        data_frame = frame_factory.build_data_frame(data_to_send)
        c.receive_data(data_frame.serialize())

        rst_frame = frame_factory.build_rst_stream_frame(
            stream_id=1
        )
        c.receive_data(rst_frame.serialize())
        c.clear_outbound_data_buffer()

        if force_cleanup:
            # Check how many streams are open to force the old one to be
            # cleaned up.
            assert c.open_outbound_streams == 0

        c.acknowledge_received_data(2048, stream_id=1)

        expected = frame_factory.build_window_update_frame(
            stream_id=0, increment=2048
        )
        assert c.data_to_send() == expected.serialize()

    def test_acknowledging_streams_we_never_saw(self, frame_factory):
        """
        If the user acknowledges a stream ID we've never seen, that raises a
        NoSuchStreamError.
        """
        c = self._setup_connection_and_send_headers(frame_factory)
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.NoSuchStreamError):
            c.acknowledge_received_data(2048, stream_id=101)

    @given(integers(min_value=1025, max_value=DEFAULT_FLOW_WINDOW))
    @settings(suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_acknowledging_1024_bytes_when_empty_increments(self,
                                                            frame_factory,
                                                            increment):
        """
        If the flow control window is empty and we acknowledge 1024 bytes or
        more, we will emit a WINDOW_UPDATE frame just to move the connection
        forward.
        """
        # We need to refresh the encoder because hypothesis has a problem with
        # integrating with py.test, meaning that we use the same frame factory
        # for all tests.
        # See https://github.com/HypothesisWorks/hypothesis-python/issues/377
        frame_factory.refresh_encoder()

        c = self._setup_connection_and_send_headers(frame_factory)

        data_to_send = b'\x00' * self.DEFAULT_FLOW_WINDOW
        data_frame = frame_factory.build_data_frame(data_to_send)
        c.receive_data(data_frame.serialize())

        c.acknowledge_received_data(increment, stream_id=1)

        first_expected = frame_factory.build_window_update_frame(
            stream_id=0, increment=increment
        )
        second_expected = frame_factory.build_window_update_frame(
            stream_id=1, increment=increment
        )
        expected_data = b''.join(
            [first_expected.serialize(), second_expected.serialize()]
        )
        assert c.data_to_send() == expected_data

    # This test needs to use a lower cap, because otherwise the algo will
    # increment the stream window anyway.
    @given(integers(min_value=1025, max_value=(DEFAULT_FLOW_WINDOW // 4) - 1))
    @settings(suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_connection_only_empty(self, frame_factory, increment):
        """
        If the connection flow control window is empty, but the stream flow
        control windows aren't, and 1024 bytes or more are acknowledged by the
        user, we increment the connection window only.
        """
        # We need to refresh the encoder because hypothesis has a problem with
        # integrating with py.test, meaning that we use the same frame factory
        # for all tests.
        # See https://github.com/HypothesisWorks/hypothesis-python/issues/377
        frame_factory.refresh_encoder()

        # Here we'll use 4 streams. Set them up.
        c = self._setup_connection_and_send_headers(frame_factory)

        for stream_id in [3, 5, 7]:
            f = frame_factory.build_headers_frame(
                headers=self.example_request_headers, stream_id=stream_id
            )
            c.receive_data(f.serialize())

        # Now we send 1/4 of the connection window per stream. Annoyingly,
        # that's an odd number, so we need to round the last frame up.
        data_to_send = b'\x00' * (self.DEFAULT_FLOW_WINDOW // 4)
        for stream_id in [1, 3, 5]:
            f = frame_factory.build_data_frame(
                data_to_send, stream_id=stream_id
            )
            c.receive_data(f.serialize())

        data_to_send = b'\x00' * c.remote_flow_control_window(7)
        data_frame = frame_factory.build_data_frame(data_to_send, stream_id=7)
        c.receive_data(data_frame.serialize())

        # Ok, now the actual test.
        c.acknowledge_received_data(increment, stream_id=1)

        expected_data = frame_factory.build_window_update_frame(
            stream_id=0, increment=increment
        ).serialize()
        assert c.data_to_send() == expected_data

    @given(integers(min_value=1025, max_value=DEFAULT_FLOW_WINDOW))
    @settings(suppress_health_check=[HealthCheck.function_scoped_fixture])
    def test_mixing_update_forms(self, frame_factory, increment):
        """
        If the user mixes ackowledging data with manually incrementing windows,
        we still keep track of what's going on.
        """
        # We need to refresh the encoder because hypothesis has a problem with
        # integrating with py.test, meaning that we use the same frame factory
        # for all tests.
        # See https://github.com/HypothesisWorks/hypothesis-python/issues/377
        frame_factory.refresh_encoder()

        # Empty the flow control window.
        c = self._setup_connection_and_send_headers(frame_factory)
        data_to_send = b'\x00' * self.DEFAULT_FLOW_WINDOW
        data_frame = frame_factory.build_data_frame(data_to_send)
        c.receive_data(data_frame.serialize())

        # Manually increment the connection flow control window back to fully
        # open, but leave the stream window closed.
        c.increment_flow_control_window(
            stream_id=None, increment=self.DEFAULT_FLOW_WINDOW
        )
        c.clear_outbound_data_buffer()

        # Now, acknowledge the receipt of that data. This should cause the
        # stream window to be widened, but not the connection window, because
        # it is already open.
        c.acknowledge_received_data(increment, stream_id=1)

        # We expect to see one window update frame only, for the stream.
        expected_data = frame_factory.build_window_update_frame(
            stream_id=1, increment=increment
        ).serialize()
        assert c.data_to_send() == expected_data
