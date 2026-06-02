# -*- coding: utf-8 -*-
"""
test_state_machines
~~~~~~~~~~~~~~~~~~~

These tests validate the state machines directly. Writing meaningful tests for
this case can be tricky, so the majority of these tests use Hypothesis to try
to talk about general behaviours rather than specific cases.
"""
import pytest

import h2.connection
import h2.exceptions
import h2.stream

from hypothesis import given
from hypothesis.strategies import sampled_from


class TestConnectionStateMachine(object):
    """
    Tests of the connection state machine.
    """
    @given(state=sampled_from(h2.connection.ConnectionState),
           input_=sampled_from(h2.connection.ConnectionInputs))
    def test_state_transitions(self, state, input_):
        c = h2.connection.H2ConnectionStateMachine()
        c.state = state

        try:
            c.process_input(input_)
        except h2.exceptions.ProtocolError:
            assert c.state == h2.connection.ConnectionState.CLOSED
        else:
            assert c.state in h2.connection.ConnectionState

    def test_state_machine_only_allows_connection_states(self):
        """
        The Connection state machine only allows ConnectionState inputs.
        """
        c = h2.connection.H2ConnectionStateMachine()

        with pytest.raises(ValueError):
            c.process_input(1)

    @pytest.mark.parametrize(
        "state",
        (
            s for s in h2.connection.ConnectionState
            if s != h2.connection.ConnectionState.CLOSED
        ),
    )
    @pytest.mark.parametrize(
        "input_",
        [
            h2.connection.ConnectionInputs.RECV_PRIORITY,
            h2.connection.ConnectionInputs.SEND_PRIORITY
        ]
    )
    def test_priority_frames_allowed_in_all_states(self, state, input_):
        """
        Priority frames can be sent/received in all connection states except
        closed.
        """
        c = h2.connection.H2ConnectionStateMachine()
        c.state = state

        c.process_input(input_)


class TestStreamStateMachine(object):
    """
    Tests of the stream state machine.
    """
    @given(state=sampled_from(h2.stream.StreamState),
           input_=sampled_from(h2.stream.StreamInputs))
    def test_state_transitions(self, state, input_):
        s = h2.stream.H2StreamStateMachine(stream_id=1)
        s.state = state

        try:
            s.process_input(input_)
        except h2.exceptions.StreamClosedError:
            # This can only happen for streams that started in the closed
            # state OR where the input was RECV_DATA and the state was not
            # OPEN or HALF_CLOSED_LOCAL OR where the state was
            # HALF_CLOSED_REMOTE and a frame was received.
            if state == h2.stream.StreamState.CLOSED:
                assert s.state == h2.stream.StreamState.CLOSED
            elif input_ == h2.stream.StreamInputs.RECV_DATA:
                assert s.state == h2.stream.StreamState.CLOSED
                assert state not in (
                    h2.stream.StreamState.OPEN,
                    h2.stream.StreamState.HALF_CLOSED_LOCAL,
                )
            elif state == h2.stream.StreamState.HALF_CLOSED_REMOTE:
                assert input_ in (
                    h2.stream.StreamInputs.RECV_HEADERS,
                    h2.stream.StreamInputs.RECV_PUSH_PROMISE,
                    h2.stream.StreamInputs.RECV_DATA,
                    h2.stream.StreamInputs.RECV_CONTINUATION,
                )
        except h2.exceptions.ProtocolError:
            assert s.state == h2.stream.StreamState.CLOSED
        else:
            assert s.state in h2.stream.StreamState

    def test_state_machine_only_allows_stream_states(self):
        """
        The Stream state machine only allows StreamState inputs.
        """
        s = h2.stream.H2StreamStateMachine(stream_id=1)

        with pytest.raises(ValueError):
            s.process_input(1)

    def test_stream_state_machine_forbids_pushes_on_server_streams(self):
        """
        Streams where this peer is a server do not allow receiving pushed
        frames.
        """
        s = h2.stream.H2StreamStateMachine(stream_id=1)
        s.process_input(h2.stream.StreamInputs.RECV_HEADERS)

        with pytest.raises(h2.exceptions.ProtocolError):
            s.process_input(h2.stream.StreamInputs.RECV_PUSH_PROMISE)

    def test_stream_state_machine_forbids_sending_pushes_from_clients(self):
        """
        Streams where this peer is a client do not allow sending pushed frames.
        """
        s = h2.stream.H2StreamStateMachine(stream_id=1)
        s.process_input(h2.stream.StreamInputs.SEND_HEADERS)

        with pytest.raises(h2.exceptions.ProtocolError):
            s.process_input(h2.stream.StreamInputs.SEND_PUSH_PROMISE)

    @pytest.mark.parametrize(
        "input_",
        [
            h2.stream.StreamInputs.SEND_HEADERS,
            h2.stream.StreamInputs.SEND_PUSH_PROMISE,
            h2.stream.StreamInputs.SEND_RST_STREAM,
            h2.stream.StreamInputs.SEND_DATA,
            h2.stream.StreamInputs.SEND_WINDOW_UPDATE,
            h2.stream.StreamInputs.SEND_END_STREAM,
        ]
    )
    def test_cannot_send_on_closed_streams(self, input_):
        """
        Sending anything but a PRIORITY frame is forbidden on closed streams.
        """
        c = h2.stream.H2StreamStateMachine(stream_id=1)
        c.state = h2.stream.StreamState.CLOSED

        expected_error = (
            h2.exceptions.ProtocolError
            if input_ == h2.stream.StreamInputs.SEND_PUSH_PROMISE
            else h2.exceptions.StreamClosedError
        )

        with pytest.raises(expected_error):
            c.process_input(input_)
