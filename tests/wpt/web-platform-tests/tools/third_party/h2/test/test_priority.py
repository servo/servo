# -*- coding: utf-8 -*-
"""
test_priority
~~~~~~~~~~~~~

Test the priority logic of Hyper-h2.
"""
import pytest

import h2.config
import h2.connection
import h2.errors
import h2.events
import h2.exceptions
import h2.stream


class TestPriority(object):
    """
    Basic priority tests.
    """
    example_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'GET'),
    ]
    example_response_headers = [
        (':status', '200'),
        ('server', 'pytest-h2'),
    ]
    server_config = h2.config.H2Configuration(client_side=False)

    def test_receiving_priority_emits_priority_update(self, frame_factory):
        """
        Receiving a priority frame emits a PriorityUpdated event.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_priority_frame(
            stream_id=1,
            weight=255,
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 1
        assert not c.data_to_send()

        event = events[0]
        assert isinstance(event, h2.events.PriorityUpdated)
        assert event.stream_id == 1
        assert event.depends_on == 0
        assert event.weight == 256
        assert event.exclusive is False

    def test_headers_with_priority_info(self, frame_factory):
        """
        Receiving a HEADERS frame with priority information on it emits a
        PriorityUpdated event.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers,
            stream_id=3,
            flags=['PRIORITY'],
            stream_weight=15,
            depends_on=1,
            exclusive=True,
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 2
        assert not c.data_to_send()

        event = events[1]
        assert isinstance(event, h2.events.PriorityUpdated)
        assert event.stream_id == 3
        assert event.depends_on == 1
        assert event.weight == 16
        assert event.exclusive is True

    def test_streams_may_not_depend_on_themselves(self, frame_factory):
        """
        A stream adjusted to depend on itself causes a Protocol Error.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers,
            stream_id=3,
            flags=['PRIORITY'],
            stream_weight=15,
            depends_on=1,
            exclusive=True,
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_priority_frame(
            stream_id=3,
            depends_on=3,
            weight=15
        )
        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(f.serialize())

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=3,
            error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR,
        )
        assert c.data_to_send() == expected_frame.serialize()

    @pytest.mark.parametrize(
        'depends_on,weight,exclusive',
        [
            (0, 256, False),
            (3, 128, False),
            (3, 128, True),
        ]
    )
    def test_can_prioritize_stream(self, depends_on, weight, exclusive,
                                   frame_factory):
        """
        hyper-h2 can emit priority frames.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        c.send_headers(headers=self.example_request_headers, stream_id=1)
        c.send_headers(headers=self.example_request_headers, stream_id=3)
        c.clear_outbound_data_buffer()

        c.prioritize(
            stream_id=1,
            depends_on=depends_on,
            weight=weight,
            exclusive=exclusive
        )

        f = frame_factory.build_priority_frame(
            stream_id=1,
            weight=weight - 1,
            depends_on=depends_on,
            exclusive=exclusive,
        )
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize(
        'depends_on,weight,exclusive',
        [
            (0, 256, False),
            (1, 128, False),
            (1, 128, True),
        ]
    )
    def test_emit_headers_with_priority_info(self, depends_on, weight,
                                             exclusive, frame_factory):
        """
        It is possible to send a headers frame with priority information on
        it.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.clear_outbound_data_buffer()

        c.send_headers(
            headers=self.example_request_headers,
            stream_id=3,
            priority_weight=weight,
            priority_depends_on=depends_on,
            priority_exclusive=exclusive,
        )

        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers,
            stream_id=3,
            flags=['PRIORITY'],
            stream_weight=weight - 1,
            depends_on=depends_on,
            exclusive=exclusive,
        )
        assert c.data_to_send() == f.serialize()

    def test_may_not_prioritize_stream_to_depend_on_self(self, frame_factory):
        """
        A stream adjusted to depend on itself causes a Protocol Error.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.send_headers(
            headers=self.example_request_headers,
            stream_id=3,
            priority_weight=255,
            priority_depends_on=0,
            priority_exclusive=False,
        )
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.prioritize(
                stream_id=3,
                depends_on=3,
            )

        assert not c.data_to_send()

    def test_may_not_initially_set_stream_depend_on_self(self, frame_factory):
        """
        A stream that starts by depending on itself causes a Protocol Error.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.send_headers(
                headers=self.example_request_headers,
                stream_id=3,
                priority_depends_on=3,
            )

        assert not c.data_to_send()

    @pytest.mark.parametrize('weight', [0, -15, 257])
    def test_prioritize_requires_valid_weight(self, weight):
        """
        A call to prioritize with an invalid weight causes a ProtocolError.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.prioritize(stream_id=1, weight=weight)

        assert not c.data_to_send()

    @pytest.mark.parametrize('weight', [0, -15, 257])
    def test_send_headers_requires_valid_weight(self, weight):
        """
        A call to send_headers with an invalid weight causes a ProtocolError.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.send_headers(
                stream_id=1,
                headers=self.example_request_headers,
                priority_weight=weight
            )

        assert not c.data_to_send()

    def test_prioritize_defaults(self, frame_factory):
        """
        When prioritize() is called with no explicit arguments, it emits a
        weight of 16, depending on stream zero non-exclusively.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.clear_outbound_data_buffer()

        c.prioritize(stream_id=1)

        f = frame_factory.build_priority_frame(
            stream_id=1,
            weight=15,
            depends_on=0,
            exclusive=False,
        )
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize(
        'priority_kwargs',
        [
            {'priority_weight': 16},
            {'priority_depends_on': 0},
            {'priority_exclusive': False},
        ]
    )
    def test_send_headers_defaults(self, priority_kwargs, frame_factory):
        """
        When send_headers() is called with only one explicit argument, it emits
        default values for everything else.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.clear_outbound_data_buffer()

        c.send_headers(
            stream_id=1,
            headers=self.example_request_headers,
            **priority_kwargs
        )

        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers,
            stream_id=1,
            flags=['PRIORITY'],
            stream_weight=15,
            depends_on=0,
            exclusive=False,
        )
        assert c.data_to_send() == f.serialize()

    def test_servers_cannot_prioritize(self, frame_factory):
        """
        Server stacks are not allowed to call ``prioritize()``.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.example_request_headers,
        )
        c.receive_data(f.serialize())

        with pytest.raises(h2.exceptions.RFC1122Error):
            c.prioritize(stream_id=1)

    def test_servers_cannot_prioritize_with_headers(self, frame_factory):
        """
        Server stacks are not allowed to prioritize on headers either.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.example_request_headers,
        )
        c.receive_data(f.serialize())

        with pytest.raises(h2.exceptions.RFC1122Error):
            c.send_headers(
                stream_id=1,
                headers=self.example_response_headers,
                priority_weight=16,
            )
