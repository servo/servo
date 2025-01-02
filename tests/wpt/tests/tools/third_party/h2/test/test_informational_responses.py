# -*- coding: utf-8 -*-
"""
test_informational_responses
~~~~~~~~~~~~~~~~~~~~~~~~~~

Tests that validate that hyper-h2 correctly handles informational (1XX)
responses in its state machine.
"""
import pytest

import h2.config
import h2.connection
import h2.events
import h2.exceptions


class TestReceivingInformationalResponses(object):
    """
    Tests for receiving informational responses.
    """
    example_request_headers = [
        (b':authority', b'example.com'),
        (b':path', b'/'),
        (b':scheme', b'https'),
        (b':method', b'GET'),
        (b'expect', b'100-continue'),
    ]
    example_informational_headers = [
        (b':status', b'100'),
        (b'server', b'fake-serv/0.1.0')
    ]
    example_response_headers = [
        (b':status', b'200'),
        (b'server', b'fake-serv/0.1.0')
    ]
    example_trailers = [
        (b'trailer', b'you-bet'),
    ]

    @pytest.mark.parametrize('end_stream', (True, False))
    def test_single_informational_response(self, frame_factory, end_stream):
        """
        When receiving a informational response, the appropriate event is
        signaled.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1,
            headers=self.example_request_headers,
            end_stream=end_stream
        )

        f = frame_factory.build_headers_frame(
            headers=self.example_informational_headers,
            stream_id=1,
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.InformationalResponseReceived)
        assert event.headers == self.example_informational_headers
        assert event.stream_id == 1

    @pytest.mark.parametrize('end_stream', (True, False))
    def test_receiving_multiple_header_blocks(self, frame_factory, end_stream):
        """
        At least three header blocks can be received: informational, headers,
        trailers.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1,
            headers=self.example_request_headers,
            end_stream=end_stream
        )

        f1 = frame_factory.build_headers_frame(
            headers=self.example_informational_headers,
            stream_id=1,
        )
        f2 = frame_factory.build_headers_frame(
            headers=self.example_response_headers,
            stream_id=1,
        )
        f3 = frame_factory.build_headers_frame(
            headers=self.example_trailers,
            stream_id=1,
            flags=['END_STREAM'],
        )
        events = c.receive_data(
            f1.serialize() + f2.serialize() + f3.serialize()
        )

        assert len(events) == 4

        assert isinstance(events[0], h2.events.InformationalResponseReceived)
        assert events[0].headers == self.example_informational_headers
        assert events[0].stream_id == 1

        assert isinstance(events[1], h2.events.ResponseReceived)
        assert events[1].headers == self.example_response_headers
        assert events[1].stream_id == 1

        assert isinstance(events[2], h2.events.TrailersReceived)
        assert events[2].headers == self.example_trailers
        assert events[2].stream_id == 1

    @pytest.mark.parametrize('end_stream', (True, False))
    def test_receiving_multiple_informational_responses(self,
                                                        frame_factory,
                                                        end_stream):
        """
        More than one informational response is allowed.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1,
            headers=self.example_request_headers,
            end_stream=end_stream
        )

        f1 = frame_factory.build_headers_frame(
            headers=self.example_informational_headers,
            stream_id=1,
        )
        f2 = frame_factory.build_headers_frame(
            headers=[(':status', '101')],
            stream_id=1,
        )
        events = c.receive_data(f1.serialize() + f2.serialize())

        assert len(events) == 2

        assert isinstance(events[0], h2.events.InformationalResponseReceived)
        assert events[0].headers == self.example_informational_headers
        assert events[0].stream_id == 1

        assert isinstance(events[1], h2.events.InformationalResponseReceived)
        assert events[1].headers == [(b':status', b'101')]
        assert events[1].stream_id == 1

    @pytest.mark.parametrize('end_stream', (True, False))
    def test_receive_provisional_response_with_end_stream(self,
                                                          frame_factory,
                                                          end_stream):
        """
        Receiving provisional responses with END_STREAM set causes
        ProtocolErrors.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1,
            headers=self.example_request_headers,
            end_stream=end_stream
        )
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            headers=self.example_informational_headers,
            stream_id=1,
            flags=['END_STREAM']
        )

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

        expected = frame_factory.build_goaway_frame(
            last_stream_id=0,
            error_code=1,
        )
        assert c.data_to_send() == expected.serialize()

    @pytest.mark.parametrize('end_stream', (True, False))
    def test_receiving_out_of_order_headers(self, frame_factory, end_stream):
        """
        When receiving a informational response after the actual response
        headers we consider it a ProtocolError and raise it.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1,
            headers=self.example_request_headers,
            end_stream=end_stream
        )

        f1 = frame_factory.build_headers_frame(
            headers=self.example_response_headers,
            stream_id=1,
        )
        f2 = frame_factory.build_headers_frame(
            headers=self.example_informational_headers,
            stream_id=1,
        )
        c.receive_data(f1.serialize())
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f2.serialize())

        expected = frame_factory.build_goaway_frame(
            last_stream_id=0,
            error_code=1,
        )
        assert c.data_to_send() == expected.serialize()


class TestSendingInformationalResponses(object):
    """
    Tests for sending informational responses.
    """
    example_request_headers = [
        (b':authority', b'example.com'),
        (b':path', b'/'),
        (b':scheme', b'https'),
        (b':method', b'GET'),
        (b'expect', b'100-continue'),
    ]
    unicode_informational_headers = [
        (u':status', u'100'),
        (u'server', u'fake-serv/0.1.0')
    ]
    bytes_informational_headers = [
        (b':status', b'100'),
        (b'server', b'fake-serv/0.1.0')
    ]
    example_response_headers = [
        (b':status', b'200'),
        (b'server', b'fake-serv/0.1.0')
    ]
    example_trailers = [
        (b'trailer', b'you-bet'),
    ]
    server_config = h2.config.H2Configuration(client_side=False)

    @pytest.mark.parametrize(
        'hdrs', (unicode_informational_headers, bytes_informational_headers),
    )
    @pytest.mark.parametrize('end_stream', (True, False))
    def test_single_informational_response(self,
                                           frame_factory,
                                           hdrs,
                                           end_stream):
        """
        When sending a informational response, the appropriate frames are
        emitted.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        flags = ['END_STREAM'] if end_stream else []
        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers,
            stream_id=1,
            flags=flags,
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()
        frame_factory.refresh_encoder()

        c.send_headers(
            stream_id=1,
            headers=hdrs
        )

        f = frame_factory.build_headers_frame(
            headers=hdrs,
            stream_id=1,
        )
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize(
        'hdrs', (unicode_informational_headers, bytes_informational_headers),
    )
    @pytest.mark.parametrize('end_stream', (True, False))
    def test_sending_multiple_header_blocks(self,
                                            frame_factory,
                                            hdrs,
                                            end_stream):
        """
        At least three header blocks can be sent: informational, headers,
        trailers.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        flags = ['END_STREAM'] if end_stream else []
        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers,
            stream_id=1,
            flags=flags,
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()
        frame_factory.refresh_encoder()

        # Send the three header blocks.
        c.send_headers(
            stream_id=1,
            headers=hdrs
        )
        c.send_headers(
            stream_id=1,
            headers=self.example_response_headers
        )
        c.send_headers(
            stream_id=1,
            headers=self.example_trailers,
            end_stream=True
        )

        # Check that we sent them properly.
        f1 = frame_factory.build_headers_frame(
            headers=hdrs,
            stream_id=1,
        )
        f2 = frame_factory.build_headers_frame(
            headers=self.example_response_headers,
            stream_id=1,
        )
        f3 = frame_factory.build_headers_frame(
            headers=self.example_trailers,
            stream_id=1,
            flags=['END_STREAM']
        )
        assert (
            c.data_to_send() ==
            f1.serialize() + f2.serialize() + f3.serialize()
        )

    @pytest.mark.parametrize(
        'hdrs', (unicode_informational_headers, bytes_informational_headers),
    )
    @pytest.mark.parametrize('end_stream', (True, False))
    def test_sending_multiple_informational_responses(self,
                                                      frame_factory,
                                                      hdrs,
                                                      end_stream):
        """
        More than one informational response is allowed.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        flags = ['END_STREAM'] if end_stream else []
        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers,
            stream_id=1,
            flags=flags,
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()
        frame_factory.refresh_encoder()

        # Send two informational responses.
        c.send_headers(
            stream_id=1,
            headers=hdrs,
        )
        c.send_headers(
            stream_id=1,
            headers=[(':status', '101')]
        )

        # Check we sent them both.
        f1 = frame_factory.build_headers_frame(
            headers=hdrs,
            stream_id=1,
        )
        f2 = frame_factory.build_headers_frame(
            headers=[(':status', '101')],
            stream_id=1,
        )
        assert c.data_to_send() == f1.serialize() + f2.serialize()

    @pytest.mark.parametrize(
        'hdrs', (unicode_informational_headers, bytes_informational_headers),
    )
    @pytest.mark.parametrize('end_stream', (True, False))
    def test_send_provisional_response_with_end_stream(self,
                                                       frame_factory,
                                                       hdrs,
                                                       end_stream):
        """
        Sending provisional responses with END_STREAM set causes
        ProtocolErrors.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        flags = ['END_STREAM'] if end_stream else []
        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers,
            stream_id=1,
            flags=flags,
        )
        c.receive_data(f.serialize())

        with pytest.raises(h2.exceptions.ProtocolError):
            c.send_headers(
                stream_id=1,
                headers=hdrs,
                end_stream=True,
            )

    @pytest.mark.parametrize(
        'hdrs', (unicode_informational_headers, bytes_informational_headers),
    )
    @pytest.mark.parametrize('end_stream', (True, False))
    def test_reject_sending_out_of_order_headers(self,
                                                 frame_factory,
                                                 hdrs,
                                                 end_stream):
        """
        When sending an informational response after the actual response
        headers we consider it a ProtocolError and raise it.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        flags = ['END_STREAM'] if end_stream else []
        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers,
            stream_id=1,
            flags=flags,
        )
        c.receive_data(f.serialize())

        c.send_headers(
            stream_id=1,
            headers=self.example_response_headers
        )

        with pytest.raises(h2.exceptions.ProtocolError):
            c.send_headers(
                stream_id=1,
                headers=hdrs
            )
