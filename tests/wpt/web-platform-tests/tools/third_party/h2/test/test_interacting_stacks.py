# -*- coding: utf-8 -*-
"""
test_interacting_stacks
~~~~~~~~~~~~~~~~~~~~~~~

These tests run two entities, a client and a server, in parallel threads. These
two entities talk to each other, running what amounts to a number of carefully
controlled simulations of real flows.

This is to ensure that the stack as a whole behaves intelligently in both
client and server cases.

These tests are long, complex, and somewhat brittle, so they aren't in general
recommended for writing the majority of test cases. Their purposes is primarily
to validate that the top-level API of the library behaves as described.

We should also consider writing helper functions to reduce the complexity of
these tests, so that they can be written more easily, as they are remarkably
useful.
"""
import coroutine_tests

import h2.config
import h2.connection
import h2.events
import h2.settings


class TestCommunication(coroutine_tests.CoroutineTestCase):
    """
    Test that two communicating state machines can work together.
    """
    server_config = h2.config.H2Configuration(client_side=False)

    def test_basic_request_response(self):
        """
        A request issued by hyper-h2 can be responded to by hyper-h2.
        """
        request_headers = [
            (b':method', b'GET'),
            (b':path', b'/'),
            (b':authority', b'example.com'),
            (b':scheme', b'https'),
            (b'user-agent', b'test-client/0.1.0'),
        ]
        response_headers = [
            (b':status', b'204'),
            (b'server', b'test-server/0.1.0'),
            (b'content-length', b'0'),
        ]

        def client():
            c = h2.connection.H2Connection()

            # Do the handshake. First send the preamble.
            c.initiate_connection()
            data = yield c.data_to_send()

            # Next, handle the remote preamble.
            events = c.receive_data(data)
            assert len(events) == 2
            assert isinstance(events[0], h2.events.SettingsAcknowledged)
            assert isinstance(events[1], h2.events.RemoteSettingsChanged)
            changed = events[1].changed_settings
            assert (
                changed[
                    h2.settings.SettingCodes.MAX_CONCURRENT_STREAMS
                ].new_value == 100
            )

            # Send a request.
            events = c.send_headers(1, request_headers, end_stream=True)
            assert not events
            data = yield c.data_to_send()

            # Validate the response.
            events = c.receive_data(data)
            assert len(events) == 2
            assert isinstance(events[0], h2.events.ResponseReceived)
            assert events[0].stream_id == 1
            assert events[0].headers == response_headers
            assert isinstance(events[1], h2.events.StreamEnded)
            assert events[1].stream_id == 1

        @self.server
        def server():
            c = h2.connection.H2Connection(config=self.server_config)

            # First, read for the preamble.
            data = yield
            events = c.receive_data(data)
            assert len(events) == 1
            assert isinstance(events[0], h2.events.RemoteSettingsChanged)
            changed = events[0].changed_settings
            assert (
                changed[
                    h2.settings.SettingCodes.MAX_CONCURRENT_STREAMS
                ].new_value == 100
            )

            # Send our preamble back.
            c.initiate_connection()
            data = yield c.data_to_send()

            # Listen for the request.
            events = c.receive_data(data)
            assert len(events) == 3
            assert isinstance(events[0], h2.events.SettingsAcknowledged)
            assert isinstance(events[1], h2.events.RequestReceived)
            assert events[1].stream_id == 1
            assert events[1].headers == request_headers
            assert isinstance(events[2], h2.events.StreamEnded)
            assert events[2].stream_id == 1

            # Send our response.
            events = c.send_headers(1, response_headers, end_stream=True)
            assert not events
            yield c.data_to_send()

        self.run_until_complete(client(), server())
