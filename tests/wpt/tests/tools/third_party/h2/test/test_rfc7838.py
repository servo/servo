# -*- coding: utf-8 -*-
"""
test_rfc7838
~~~~~~~~~~~~

Test the RFC 7838 ALTSVC support.
"""
import pytest

import h2.config
import h2.connection
import h2.events
import h2.exceptions


class TestRFC7838Client(object):
    """
    Tests that the client supports receiving the RFC 7838 AltSvc frame.
    """
    example_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'GET'),
    ]
    example_response_headers = [
        (u':status', u'200'),
        (u'server', u'fake-serv/0.1.0')
    ]

    def test_receiving_altsvc_stream_zero(self, frame_factory):
        """
        An ALTSVC frame received on stream zero correctly transposes all the
        fields from the frames.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=0, origin=b"example.com", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.AlternativeServiceAvailable)
        assert event.origin == b"example.com"
        assert event.field_value == b'h2=":8000"; ma=60'

        # No data gets sent.
        assert not c.data_to_send()

    def test_receiving_altsvc_stream_zero_no_origin(self, frame_factory):
        """
        An ALTSVC frame received on stream zero without an origin field is
        ignored.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=0, origin=b"", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert not events
        assert not c.data_to_send()

    def test_receiving_altsvc_on_stream(self, frame_factory):
        """
        An ALTSVC frame received on a stream correctly transposes all the
        fields from the frame and attaches the expected origin.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=1, origin=b"", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.AlternativeServiceAvailable)
        assert event.origin == b"example.com"
        assert event.field_value == b'h2=":8000"; ma=60'

        # No data gets sent.
        assert not c.data_to_send()

    def test_receiving_altsvc_on_stream_with_origin(self, frame_factory):
        """
        An ALTSVC frame received on a stream with an origin field present gets
        ignored.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=1, origin=b"example.com", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 0
        assert not c.data_to_send()

    def test_receiving_altsvc_on_stream_not_yet_opened(self, frame_factory):
        """
        When an ALTSVC frame is received on a stream the client hasn't yet
        opened, the frame is ignored.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.clear_outbound_data_buffer()

        # We'll test this twice, once on a client-initiated stream ID and once
        # on a server initiated one.
        f1 = frame_factory.build_alt_svc_frame(
            stream_id=1, origin=b"", field=b'h2=":8000"; ma=60'
        )
        f2 = frame_factory.build_alt_svc_frame(
            stream_id=2, origin=b"", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f1.serialize() + f2.serialize())

        assert len(events) == 0
        assert not c.data_to_send()

    def test_receiving_altsvc_before_sending_headers(self, frame_factory):
        """
        When an ALTSVC frame is received but the client hasn't sent headers yet
        it gets ignored.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        # We need to create the idle stream. We have to do it by calling
        # a private API. While this can't naturally happen in hyper-h2 (we
        # don't currently have a mechanism by which this could occur), it could
        # happen in the future and we defend against it.
        c._begin_new_stream(
            stream_id=1, allowed_ids=h2.connection.AllowedStreamIDs.ODD
        )
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=1, origin=b"", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 0
        assert not c.data_to_send()

    def test_receiving_altsvc_after_receiving_headers(self, frame_factory):
        """
        When an ALTSVC frame is received but the server has already sent
        headers it gets ignored.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)

        f = frame_factory.build_headers_frame(
            headers=self.example_response_headers
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=1, origin=b"", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 0
        assert not c.data_to_send()

    def test_receiving_altsvc_on_closed_stream(self, frame_factory):
        """
        When an ALTSVC frame is received on a closed stream, we ignore it.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1, headers=self.example_request_headers, end_stream=True
        )

        f = frame_factory.build_headers_frame(
            headers=self.example_response_headers,
            flags=['END_STREAM'],
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=1, origin=b"", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 0
        assert not c.data_to_send()

    def test_receiving_altsvc_on_pushed_stream(self, frame_factory):
        """
        When an ALTSVC frame is received on a stream that the server pushed,
        the frame is accepted.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)

        f = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=self.example_request_headers
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=2, origin=b"", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 1
        event = events[0]

        assert isinstance(event, h2.events.AlternativeServiceAvailable)
        assert event.origin == b"example.com"
        assert event.field_value == b'h2=":8000"; ma=60'

        # No data gets sent.
        assert not c.data_to_send()

    def test_cannot_send_explicit_alternative_service(self, frame_factory):
        """
        A client cannot send an explicit alternative service.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.advertise_alternative_service(
                field_value=b'h2=":8000"; ma=60',
                origin=b"example.com",
            )

    def test_cannot_send_implicit_alternative_service(self, frame_factory):
        """
        A client cannot send an implicit alternative service.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(stream_id=1, headers=self.example_request_headers)
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.advertise_alternative_service(
                field_value=b'h2=":8000"; ma=60',
                stream_id=1,
            )


class TestRFC7838Server(object):
    """
    Tests that the server supports sending the RFC 7838 AltSvc frame.
    """
    example_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'GET'),
    ]
    example_response_headers = [
        (u':status', u'200'),
        (u'server', u'fake-serv/0.1.0')
    ]

    server_config = h2.config.H2Configuration(client_side=False)

    def test_receiving_altsvc_as_server_stream_zero(self, frame_factory):
        """
        When an ALTSVC frame is received on stream zero and we are a server,
        we ignore it.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=0, origin=b"example.com", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 0
        assert not c.data_to_send()

    def test_receiving_altsvc_as_server_on_stream(self, frame_factory):
        """
        When an ALTSVC frame is received on a stream and we are a server, we
        ignore it.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_alt_svc_frame(
            stream_id=1, origin=b"", field=b'h2=":8000"; ma=60'
        )
        events = c.receive_data(f.serialize())

        assert len(events) == 0
        assert not c.data_to_send()

    def test_sending_explicit_alternative_service(self, frame_factory):
        """
        A server can send an explicit alternative service.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        c.advertise_alternative_service(
            field_value=b'h2=":8000"; ma=60',
            origin=b"example.com",
        )

        f = frame_factory.build_alt_svc_frame(
            stream_id=0, origin=b"example.com", field=b'h2=":8000"; ma=60'
        )
        assert c.data_to_send() == f.serialize()

    def test_sending_implicit_alternative_service(self, frame_factory):
        """
        A server can send an implicit alternative service.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )
        c.receive_data(f.serialize())
        c.clear_outbound_data_buffer()

        c.advertise_alternative_service(
            field_value=b'h2=":8000"; ma=60',
            stream_id=1,
        )

        f = frame_factory.build_alt_svc_frame(
            stream_id=1, origin=b"", field=b'h2=":8000"; ma=60'
        )
        assert c.data_to_send() == f.serialize()

    def test_no_implicit_alternative_service_before_headers(self,
                                                            frame_factory):
        """
        If headers haven't been received yet, the server forbids sending an
        implicit alternative service.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.advertise_alternative_service(
                field_value=b'h2=":8000"; ma=60',
                stream_id=1,
            )

    def test_no_implicit_alternative_service_after_response(self,
                                                            frame_factory):
        """
        If the server has sent response headers, hyper-h2 forbids sending an
        implicit alternative service.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )
        c.receive_data(f.serialize())
        c.send_headers(stream_id=1, headers=self.example_response_headers)
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.advertise_alternative_service(
                field_value=b'h2=":8000"; ma=60',
                stream_id=1,
            )

    def test_cannot_provide_origin_and_stream_id(self, frame_factory):
        """
        The user cannot provide both the origin and stream_id arguments when
        advertising alternative services.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        f = frame_factory.build_headers_frame(
            headers=self.example_request_headers
        )
        c.receive_data(f.serialize())

        with pytest.raises(ValueError):
            c.advertise_alternative_service(
                field_value=b'h2=":8000"; ma=60',
                origin=b"example.com",
                stream_id=1,
            )

    def test_cannot_provide_unicode_altsvc_field(self, frame_factory):
        """
        The user cannot provide the field value for alternative services as a
        unicode string.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        with pytest.raises(ValueError):
            c.advertise_alternative_service(
                field_value=u'h2=":8000"; ma=60',
                origin=b"example.com",
            )
