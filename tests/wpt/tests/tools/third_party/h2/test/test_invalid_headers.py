# -*- coding: utf-8 -*-
"""
test_invalid_headers.py
~~~~~~~~~~~~~~~~~~~~~~~

This module contains tests that use invalid header blocks, and validates that
they fail appropriately.
"""
import itertools

import pytest

import h2.config
import h2.connection
import h2.errors
import h2.events
import h2.exceptions
import h2.settings
import h2.utilities

import hyperframe.frame

from hypothesis import given
from hypothesis.strategies import binary, lists, tuples

HEADERS_STRATEGY = lists(tuples(binary(min_size=1), binary()))


class TestInvalidFrameSequences(object):
    """
    Invalid header sequences cause ProtocolErrors to be thrown when received.
    """
    base_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'GET'),
        ('user-agent', 'someua/0.0.1'),
    ]
    invalid_header_blocks = [
        base_request_headers + [('Uppercase', 'name')],
        base_request_headers + [(':late', 'pseudo-header')],
        [(':path', 'duplicate-pseudo-header')] + base_request_headers,
        base_request_headers + [('connection', 'close')],
        base_request_headers + [('proxy-connection', 'close')],
        base_request_headers + [('keep-alive', 'close')],
        base_request_headers + [('transfer-encoding', 'gzip')],
        base_request_headers + [('upgrade', 'super-protocol/1.1')],
        base_request_headers + [('te', 'chunked')],
        base_request_headers + [('host', 'notexample.com')],
        base_request_headers + [(' name', 'name with leading space')],
        base_request_headers + [('name ', 'name with trailing space')],
        base_request_headers + [('name', ' value with leading space')],
        base_request_headers + [('name', 'value with trailing space ')],
        [header for header in base_request_headers
         if header[0] != ':authority'],
        [(':protocol', 'websocket')] + base_request_headers,
    ]
    server_config = h2.config.H2Configuration(
        client_side=False, header_encoding='utf-8'
    )

    @pytest.mark.parametrize('headers', invalid_header_blocks)
    def test_headers_event(self, frame_factory, headers):
        """
        Test invalid headers are rejected with PROTOCOL_ERROR.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(headers)
        data = f.serialize()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(data)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1, error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()

    @pytest.mark.parametrize('headers', invalid_header_blocks)
    def test_push_promise_event(self, frame_factory, headers):
        """
        If a PUSH_PROMISE header frame is received with an invalid header block
        it is rejected with a PROTOCOL_ERROR.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1, headers=self.base_request_headers, end_stream=True
        )
        c.clear_outbound_data_buffer()

        f = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=headers
        )
        data = f.serialize()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(data)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0, error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()

    @pytest.mark.parametrize('headers', invalid_header_blocks)
    def test_push_promise_skipping_validation(self, frame_factory, headers):
        """
        If we have ``validate_inbound_headers`` disabled, then invalid header
        blocks in push promise frames are allowed to pass.
        """
        config = h2.config.H2Configuration(
            client_side=True,
            validate_inbound_headers=False,
            header_encoding='utf-8'
        )

        c = h2.connection.H2Connection(config=config)
        c.initiate_connection()
        c.send_headers(
            stream_id=1, headers=self.base_request_headers, end_stream=True
        )
        c.clear_outbound_data_buffer()

        f = frame_factory.build_push_promise_frame(
            stream_id=1,
            promised_stream_id=2,
            headers=headers
        )
        data = f.serialize()

        events = c.receive_data(data)
        assert len(events) == 1
        pp_event = events[0]
        assert pp_event.headers == headers

    @pytest.mark.parametrize('headers', invalid_header_blocks)
    def test_headers_event_skipping_validation(self, frame_factory, headers):
        """
        If we have ``validate_inbound_headers`` disabled, then all of these
        invalid header blocks are allowed to pass.
        """
        config = h2.config.H2Configuration(
            client_side=False,
            validate_inbound_headers=False,
            header_encoding='utf-8'
        )

        c = h2.connection.H2Connection(config=config)
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(headers)
        data = f.serialize()

        events = c.receive_data(data)
        assert len(events) == 1
        request_event = events[0]
        assert request_event.headers == headers

    def test_te_trailers_is_valid(self, frame_factory):
        """
        `te: trailers` is allowed by the filter.
        """
        headers = (
            self.base_request_headers + [('te', 'trailers')]
        )

        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())

        f = frame_factory.build_headers_frame(headers)
        data = f.serialize()

        events = c.receive_data(data)
        assert len(events) == 1
        request_event = events[0]
        assert request_event.headers == headers

    def test_pseudo_headers_rejected_in_trailer(self, frame_factory):
        """
        Ensure we reject pseudo headers included in trailers
        """
        trailers = [(':path', '/'), ('extra', 'value')]

        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        header_frame = frame_factory.build_headers_frame(
            self.base_request_headers
        )
        trailer_frame = frame_factory.build_headers_frame(
            trailers, flags=["END_STREAM"]
        )
        head = header_frame.serialize()
        trailer = trailer_frame.serialize()

        c.receive_data(head)
        # Raise exception if pseudo header in trailer
        with pytest.raises(h2.exceptions.ProtocolError) as e:
            c.receive_data(trailer)
        assert "pseudo-header in trailer" in str(e.value)

        # Test appropriate response frame is generated
        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1, error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()


class TestSendingInvalidFrameSequences(object):
    """
    Trying to send invalid header sequences cause ProtocolErrors to
    be thrown.
    """
    base_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'GET'),
        ('user-agent', 'someua/0.0.1'),
    ]
    invalid_header_blocks = [
        base_request_headers + [(':late', 'pseudo-header')],
        [(':path', 'duplicate-pseudo-header')] + base_request_headers,
        base_request_headers + [('te', 'chunked')],
        base_request_headers + [('host', 'notexample.com')],
        [header for header in base_request_headers
         if header[0] != ':authority'],
    ]
    strippable_header_blocks = [
        base_request_headers + [('connection', 'close')],
        base_request_headers + [('proxy-connection', 'close')],
        base_request_headers + [('keep-alive', 'close')],
        base_request_headers + [('transfer-encoding', 'gzip')],
        base_request_headers + [('upgrade', 'super-protocol/1.1')]
    ]
    all_header_blocks = invalid_header_blocks + strippable_header_blocks

    server_config = h2.config.H2Configuration(client_side=False)

    @pytest.mark.parametrize('headers', invalid_header_blocks)
    def test_headers_event(self, frame_factory, headers):
        """
        Test sending invalid headers raise a ProtocolError.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        # Clear the data, then try to send headers.
        c.clear_outbound_data_buffer()
        with pytest.raises(h2.exceptions.ProtocolError):
            c.send_headers(1, headers)

    @pytest.mark.parametrize('headers', invalid_header_blocks)
    def test_send_push_promise(self, frame_factory, headers):
        """
        Sending invalid headers in a push promise raises a ProtocolError.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        header_frame = frame_factory.build_headers_frame(
            self.base_request_headers
        )
        c.receive_data(header_frame.serialize())

        # Clear the data, then try to send a push promise.
        c.clear_outbound_data_buffer()
        with pytest.raises(h2.exceptions.ProtocolError):
            c.push_stream(
                stream_id=1, promised_stream_id=2, request_headers=headers
            )

    @pytest.mark.parametrize('headers', all_header_blocks)
    def test_headers_event_skipping_validation(self, frame_factory, headers):
        """
        If we have ``validate_outbound_headers`` disabled, then all of these
        invalid header blocks are allowed to pass.
        """
        config = h2.config.H2Configuration(
            validate_outbound_headers=False
        )

        c = h2.connection.H2Connection(config=config)
        c.initiate_connection()

        # Clear the data, then send headers.
        c.clear_outbound_data_buffer()
        c.send_headers(1, headers)

        # Ensure headers are still normalized.
        norm_headers = h2.utilities.normalize_outbound_headers(headers, None)
        f = frame_factory.build_headers_frame(norm_headers)
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize('headers', all_header_blocks)
    def test_push_promise_skipping_validation(self, frame_factory, headers):
        """
        If we have ``validate_outbound_headers`` disabled, then all of these
        invalid header blocks are allowed to pass.
        """
        config = h2.config.H2Configuration(
            client_side=False,
            validate_outbound_headers=False,
        )

        c = h2.connection.H2Connection(config=config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        header_frame = frame_factory.build_headers_frame(
            self.base_request_headers
        )
        c.receive_data(header_frame.serialize())

        # Create push promise frame with normalized headers.
        frame_factory.refresh_encoder()
        norm_headers = h2.utilities.normalize_outbound_headers(headers, None)
        pp_frame = frame_factory.build_push_promise_frame(
            stream_id=1, promised_stream_id=2, headers=norm_headers
        )

        # Clear the data, then send a push promise.
        c.clear_outbound_data_buffer()
        c.push_stream(
            stream_id=1, promised_stream_id=2, request_headers=headers
        )
        assert c.data_to_send() == pp_frame.serialize()

    @pytest.mark.parametrize('headers', all_header_blocks)
    def test_headers_event_skip_normalization(self, frame_factory, headers):
        """
        If we have ``normalize_outbound_headers`` disabled, then all of these
        invalid header blocks are sent through unmodified.
        """
        config = h2.config.H2Configuration(
            validate_outbound_headers=False,
            normalize_outbound_headers=False
        )

        c = h2.connection.H2Connection(config=config)
        c.initiate_connection()

        f = frame_factory.build_headers_frame(
            headers,
            stream_id=1,
        )

        # Clear the data, then send headers.
        c.clear_outbound_data_buffer()
        c.send_headers(1, headers)
        assert c.data_to_send() == f.serialize()

    @pytest.mark.parametrize('headers', all_header_blocks)
    def test_push_promise_skip_normalization(self, frame_factory, headers):
        """
        If we have ``normalize_outbound_headers`` disabled, then all of these
        invalid header blocks are allowed to pass unmodified.
        """
        config = h2.config.H2Configuration(
            client_side=False,
            validate_outbound_headers=False,
            normalize_outbound_headers=False,
        )

        c = h2.connection.H2Connection(config=config)
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())

        header_frame = frame_factory.build_headers_frame(
            self.base_request_headers
        )
        c.receive_data(header_frame.serialize())

        frame_factory.refresh_encoder()
        pp_frame = frame_factory.build_push_promise_frame(
            stream_id=1, promised_stream_id=2, headers=headers
        )

        # Clear the data, then send a push promise.
        c.clear_outbound_data_buffer()
        c.push_stream(
            stream_id=1, promised_stream_id=2, request_headers=headers
        )
        assert c.data_to_send() == pp_frame.serialize()

    @pytest.mark.parametrize('headers', strippable_header_blocks)
    def test_strippable_headers(self, frame_factory, headers):
        """
        Test connection related headers are removed before sending.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()

        # Clear the data, then try to send headers.
        c.clear_outbound_data_buffer()
        c.send_headers(1, headers)

        f = frame_factory.build_headers_frame(self.base_request_headers)
        assert c.data_to_send() == f.serialize()


class TestFilter(object):
    """
    Test the filter function directly.

    These tests exists to confirm the behaviour of the filter function in a
    wide range of scenarios. Many of these scenarios may not be legal for
    HTTP/2 and so may never hit the function, but it's worth validating that it
    behaves as expected anyway.
    """
    validation_functions = [
        h2.utilities.validate_headers,
        h2.utilities.validate_outbound_headers
    ]

    hdr_validation_combos = [
        h2.utilities.HeaderValidationFlags(
            is_client, is_trailer, is_response_header, is_push_promise
        )
        for is_client, is_trailer, is_response_header, is_push_promise in (
            itertools.product([True, False], repeat=4)
        )
    ]

    hdr_validation_response_headers = [
        flags for flags in hdr_validation_combos
        if flags.is_response_header
    ]

    hdr_validation_request_headers_no_trailer = [
        flags for flags in hdr_validation_combos
        if not (flags.is_trailer or flags.is_response_header)
    ]

    invalid_request_header_blocks_bytes = (
        # First, missing :method
        (
            (b':authority', b'google.com'),
            (b':path', b'/'),
            (b':scheme', b'https'),
        ),
        # Next, missing :path
        (
            (b':authority', b'google.com'),
            (b':method', b'GET'),
            (b':scheme', b'https'),
        ),
        # Next, missing :scheme
        (
            (b':authority', b'google.com'),
            (b':method', b'GET'),
            (b':path', b'/'),
        ),
        # Finally, path present but empty.
        (
            (b':authority', b'google.com'),
            (b':method', b'GET'),
            (b':scheme', b'https'),
            (b':path', b''),
        ),
    )
    invalid_request_header_blocks_unicode = (
        # First, missing :method
        (
            (':authority', 'google.com'),
            (':path', '/'),
            (':scheme', 'https'),
        ),
        # Next, missing :path
        (
            (':authority', 'google.com'),
            (':method', 'GET'),
            (':scheme', 'https'),
        ),
        # Next, missing :scheme
        (
            (':authority', 'google.com'),
            (':method', 'GET'),
            (':path', '/'),
        ),
        # Finally, path present but empty.
        (
            (':authority', 'google.com'),
            (':method', 'GET'),
            (':scheme', 'https'),
            (':path', ''),
        ),
    )

    # All headers that are forbidden from either request or response blocks.
    forbidden_request_headers_bytes = (b':status',)
    forbidden_request_headers_unicode = (':status',)
    forbidden_response_headers_bytes = (
        b':path', b':scheme', b':authority', b':method'
    )
    forbidden_response_headers_unicode = (
        ':path', ':scheme', ':authority', ':method'
    )

    @pytest.mark.parametrize('validation_function', validation_functions)
    @pytest.mark.parametrize('hdr_validation_flags', hdr_validation_combos)
    @given(headers=HEADERS_STRATEGY)
    def test_range_of_acceptable_outputs(self,
                                         headers,
                                         validation_function,
                                         hdr_validation_flags):
        """
        The header validation functions either return the data unchanged
        or throw a ProtocolError.
        """
        try:
            assert headers == list(validation_function(
                headers, hdr_validation_flags))
        except h2.exceptions.ProtocolError:
            assert True

    @pytest.mark.parametrize('hdr_validation_flags', hdr_validation_combos)
    def test_invalid_pseudo_headers(self, hdr_validation_flags):
        headers = [(b':custom', b'value')]
        with pytest.raises(h2.exceptions.ProtocolError):
            list(h2.utilities.validate_headers(headers, hdr_validation_flags))

    @pytest.mark.parametrize('validation_function', validation_functions)
    @pytest.mark.parametrize(
        'hdr_validation_flags', hdr_validation_request_headers_no_trailer
    )
    def test_matching_authority_host_headers(self,
                                             validation_function,
                                             hdr_validation_flags):
        """
        If a header block has :authority and Host headers and they match,
        the headers should pass through unchanged.
        """
        headers = [
            (b':authority', b'example.com'),
            (b':path', b'/'),
            (b':scheme', b'https'),
            (b':method', b'GET'),
            (b'host', b'example.com'),
        ]
        assert headers == list(h2.utilities.validate_headers(
            headers, hdr_validation_flags
        ))

    @pytest.mark.parametrize(
        'hdr_validation_flags', hdr_validation_response_headers
    )
    def test_response_header_without_status(self, hdr_validation_flags):
        headers = [(b'content-length', b'42')]
        with pytest.raises(h2.exceptions.ProtocolError):
            list(h2.utilities.validate_headers(headers, hdr_validation_flags))

    @pytest.mark.parametrize(
        'hdr_validation_flags', hdr_validation_request_headers_no_trailer
    )
    @pytest.mark.parametrize(
        'header_block',
        (
            invalid_request_header_blocks_bytes +
            invalid_request_header_blocks_unicode
        )
    )
    def test_outbound_req_header_missing_pseudo_headers(self,
                                                        hdr_validation_flags,
                                                        header_block):
        with pytest.raises(h2.exceptions.ProtocolError):
            list(
                h2.utilities.validate_outbound_headers(
                    header_block, hdr_validation_flags
                )
            )

    @pytest.mark.parametrize(
        'hdr_validation_flags', hdr_validation_request_headers_no_trailer
    )
    @pytest.mark.parametrize(
        'header_block', invalid_request_header_blocks_bytes
    )
    def test_inbound_req_header_missing_pseudo_headers(self,
                                                       hdr_validation_flags,
                                                       header_block):
        with pytest.raises(h2.exceptions.ProtocolError):
            list(
                h2.utilities.validate_headers(
                    header_block, hdr_validation_flags
                )
            )

    @pytest.mark.parametrize(
        'hdr_validation_flags', hdr_validation_request_headers_no_trailer
    )
    @pytest.mark.parametrize(
        'invalid_header',
        forbidden_request_headers_bytes + forbidden_request_headers_unicode
    )
    def test_outbound_req_header_extra_pseudo_headers(self,
                                                      hdr_validation_flags,
                                                      invalid_header):
        """
        Outbound request header blocks containing the forbidden request headers
        fail validation.
        """
        headers = [
            (b':path', b'/'),
            (b':scheme', b'https'),
            (b':authority', b'google.com'),
            (b':method', b'GET'),
        ]
        headers.append((invalid_header, b'some value'))
        with pytest.raises(h2.exceptions.ProtocolError):
            list(
                h2.utilities.validate_outbound_headers(
                    headers, hdr_validation_flags
                )
            )

    @pytest.mark.parametrize(
        'hdr_validation_flags', hdr_validation_request_headers_no_trailer
    )
    @pytest.mark.parametrize(
        'invalid_header',
        forbidden_request_headers_bytes
    )
    def test_inbound_req_header_extra_pseudo_headers(self,
                                                     hdr_validation_flags,
                                                     invalid_header):
        """
        Inbound request header blocks containing the forbidden request headers
        fail validation.
        """
        headers = [
            (b':path', b'/'),
            (b':scheme', b'https'),
            (b':authority', b'google.com'),
            (b':method', b'GET'),
        ]
        headers.append((invalid_header, b'some value'))
        with pytest.raises(h2.exceptions.ProtocolError):
            list(h2.utilities.validate_headers(headers, hdr_validation_flags))

    @pytest.mark.parametrize(
        'hdr_validation_flags', hdr_validation_response_headers
    )
    @pytest.mark.parametrize(
        'invalid_header',
        forbidden_response_headers_bytes + forbidden_response_headers_unicode
    )
    def test_outbound_resp_header_extra_pseudo_headers(self,
                                                       hdr_validation_flags,
                                                       invalid_header):
        """
        Outbound response header blocks containing the forbidden response
        headers fail validation.
        """
        headers = [(b':status', b'200')]
        headers.append((invalid_header, b'some value'))
        with pytest.raises(h2.exceptions.ProtocolError):
            list(
                h2.utilities.validate_outbound_headers(
                    headers, hdr_validation_flags
                )
            )

    @pytest.mark.parametrize(
        'hdr_validation_flags', hdr_validation_response_headers
    )
    @pytest.mark.parametrize(
        'invalid_header',
        forbidden_response_headers_bytes
    )
    def test_inbound_resp_header_extra_pseudo_headers(self,
                                                      hdr_validation_flags,
                                                      invalid_header):
        """
        Inbound response header blocks containing the forbidden response
        headers fail validation.
        """
        headers = [(b':status', b'200')]
        headers.append((invalid_header, b'some value'))
        with pytest.raises(h2.exceptions.ProtocolError):
            list(h2.utilities.validate_headers(headers, hdr_validation_flags))

    @pytest.mark.parametrize('hdr_validation_flags', hdr_validation_combos)
    def test_inbound_header_name_length(self, hdr_validation_flags):
        with pytest.raises(h2.exceptions.ProtocolError):
            list(h2.utilities.validate_headers([(b'', b'foobar')], hdr_validation_flags))

    def test_inbound_header_name_length_full_frame_decode(self, frame_factory):
        f = frame_factory.build_headers_frame([])
        f.data = b"\x00\x00\x05\x00\x00\x00\x00\x04"
        data = f.serialize()

        c = h2.connection.H2Connection(config=h2.config.H2Configuration(client_side=False))
        c.initiate_connection()
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        with pytest.raises(h2.exceptions.ProtocolError, match="Received header name with zero length."):
            c.receive_data(data)


class TestOversizedHeaders(object):
    """
    Tests that oversized header blocks are correctly rejected. This replicates
    the "HPACK Bomb" attack, and confirms that we're resistant against it.
    """
    request_header_block = [
        (b':method', b'GET'),
        (b':authority', b'example.com'),
        (b':scheme', b'https'),
        (b':path', b'/'),
    ]

    response_header_block = [
        (b':status', b'200'),
    ]

    # The first header block contains a single header that fills the header
    # table. To do that, we'll give it a single-character header name and a
    # 4063 byte header value. This will make it exactly the size of the header
    # table. It must come last, so that it evicts all other headers.
    # This block must be appended to either a request or response block.
    first_header_block = [
        (b'a', b'a' * 4063),
    ]

    # The second header "block" is actually a custom HEADERS frame body that
    # simply repeatedly refers to the first entry for 16kB. Each byte has the
    # high bit set (0x80), and then uses the remaining 7 bits to encode the
    # number 62 (0x3e), leading to a repeat of the byte 0xbe.
    second_header_block = b'\xbe' * 2**14

    server_config = h2.config.H2Configuration(client_side=False)

    def test_hpack_bomb_request(self, frame_factory):
        """
        A HPACK bomb request causes the connection to be torn down with the
        error code ENHANCE_YOUR_CALM.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            self.request_header_block + self.first_header_block
        )
        data = f.serialize()
        c.receive_data(data)

        # Build the attack payload.
        attack_frame = hyperframe.frame.HeadersFrame(stream_id=3)
        attack_frame.data = self.second_header_block
        attack_frame.flags.add('END_HEADERS')
        data = attack_frame.serialize()

        with pytest.raises(h2.exceptions.DenialOfServiceError):
            c.receive_data(data)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1, error_code=h2.errors.ErrorCodes.ENHANCE_YOUR_CALM
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_hpack_bomb_response(self, frame_factory):
        """
        A HPACK bomb response causes the connection to be torn down with the
        error code ENHANCE_YOUR_CALM.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1, headers=self.request_header_block
        )
        c.send_headers(
            stream_id=3, headers=self.request_header_block
        )
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            self.response_header_block + self.first_header_block
        )
        data = f.serialize()
        c.receive_data(data)

        # Build the attack payload.
        attack_frame = hyperframe.frame.HeadersFrame(stream_id=3)
        attack_frame.data = self.second_header_block
        attack_frame.flags.add('END_HEADERS')
        data = attack_frame.serialize()

        with pytest.raises(h2.exceptions.DenialOfServiceError):
            c.receive_data(data)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0, error_code=h2.errors.ErrorCodes.ENHANCE_YOUR_CALM
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_hpack_bomb_push(self, frame_factory):
        """
        A HPACK bomb push causes the connection to be torn down with the
        error code ENHANCE_YOUR_CALM.
        """
        c = h2.connection.H2Connection()
        c.initiate_connection()
        c.send_headers(
            stream_id=1, headers=self.request_header_block
        )
        c.clear_outbound_data_buffer()

        f = frame_factory.build_headers_frame(
            self.response_header_block + self.first_header_block
        )
        data = f.serialize()
        c.receive_data(data)

        # Build the attack payload. We need to shrink it by four bytes because
        # the promised_stream_id consumes four bytes of body.
        attack_frame = hyperframe.frame.PushPromiseFrame(stream_id=3)
        attack_frame.promised_stream_id = 2
        attack_frame.data = self.second_header_block[:-4]
        attack_frame.flags.add('END_HEADERS')
        data = attack_frame.serialize()

        with pytest.raises(h2.exceptions.DenialOfServiceError):
            c.receive_data(data)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=0, error_code=h2.errors.ErrorCodes.ENHANCE_YOUR_CALM
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_reject_headers_when_list_size_shrunk(self, frame_factory):
        """
        When we've shrunk the header list size, we reject new header blocks
        that violate the new size.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        # Receive the first request, which causes no problem.
        f = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.request_header_block
        )
        data = f.serialize()
        c.receive_data(data)

        # Now, send a settings change. It's un-ACKed at this time. A new
        # request arrives, also without incident.
        c.update_settings({h2.settings.SettingCodes.MAX_HEADER_LIST_SIZE: 50})
        c.clear_outbound_data_buffer()
        f = frame_factory.build_headers_frame(
            stream_id=3,
            headers=self.request_header_block
        )
        data = f.serialize()
        c.receive_data(data)

        # We get a SETTINGS ACK.
        f = frame_factory.build_settings_frame({}, ack=True)
        data = f.serialize()
        c.receive_data(data)

        # Now a third request comes in. This explodes.
        f = frame_factory.build_headers_frame(
            stream_id=5,
            headers=self.request_header_block
        )
        data = f.serialize()

        with pytest.raises(h2.exceptions.DenialOfServiceError):
            c.receive_data(data)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=3, error_code=h2.errors.ErrorCodes.ENHANCE_YOUR_CALM
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_reject_headers_when_table_size_shrunk(self, frame_factory):
        """
        When we've shrunk the header table size, we reject header blocks that
        do not respect the change.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        # Receive the first request, which causes no problem.
        f = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.request_header_block
        )
        data = f.serialize()
        c.receive_data(data)

        # Now, send a settings change. It's un-ACKed at this time. A new
        # request arrives, also without incident.
        c.update_settings({h2.settings.SettingCodes.HEADER_TABLE_SIZE: 128})
        c.clear_outbound_data_buffer()
        f = frame_factory.build_headers_frame(
            stream_id=3,
            headers=self.request_header_block
        )
        data = f.serialize()
        c.receive_data(data)

        # We get a SETTINGS ACK.
        f = frame_factory.build_settings_frame({}, ack=True)
        data = f.serialize()
        c.receive_data(data)

        # Now a third request comes in. This explodes, as it does not contain
        # a dynamic table size update.
        f = frame_factory.build_headers_frame(
            stream_id=5,
            headers=self.request_header_block
        )
        data = f.serialize()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(data)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=3, error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()

    def test_reject_headers_exceeding_table_size(self, frame_factory):
        """
        When the remote peer sends a dynamic table size update that exceeds our
        setting, we reject it.
        """
        c = h2.connection.H2Connection(config=self.server_config)
        c.receive_data(frame_factory.preamble())
        c.clear_outbound_data_buffer()

        # Receive the first request, which causes no problem.
        f = frame_factory.build_headers_frame(
            stream_id=1,
            headers=self.request_header_block
        )
        data = f.serialize()
        c.receive_data(data)

        # Now a second request comes in that sets the table size too high.
        # This explodes.
        frame_factory.change_table_size(c.local_settings.header_table_size + 1)
        f = frame_factory.build_headers_frame(
            stream_id=5,
            headers=self.request_header_block
        )
        data = f.serialize()

        with pytest.raises(h2.exceptions.ProtocolError):
            c.receive_data(data)

        expected_frame = frame_factory.build_goaway_frame(
            last_stream_id=1, error_code=h2.errors.ErrorCodes.PROTOCOL_ERROR
        )
        assert c.data_to_send() == expected_frame.serialize()
