# -*- coding: utf-8 -*-
"""
Test for the integer encoding/decoding functionality in the HPACK library.
"""
import pytest

from hypothesis import given
from hypothesis.strategies import integers, binary, one_of

from hpack import HPACKDecodingError
from hpack.hpack import encode_integer, decode_integer


class TestIntegerEncoding:
    # These tests are stolen from the HPACK spec.
    def test_encoding_10_with_5_bit_prefix(self):
        val = encode_integer(10, 5)
        assert len(val) == 1
        assert val == bytearray(b'\x0a')

    def test_encoding_1337_with_5_bit_prefix(self):
        val = encode_integer(1337, 5)
        assert len(val) == 3
        assert val == bytearray(b'\x1f\x9a\x0a')

    def test_encoding_42_with_8_bit_prefix(self):
        val = encode_integer(42, 8)
        assert len(val) == 1
        assert val == bytearray(b'\x2a')


class TestIntegerDecoding:
    # These tests are stolen from the HPACK spec.
    def test_decoding_10_with_5_bit_prefix(self):
        val = decode_integer(b'\x0a', 5)
        assert val == (10, 1)

    def test_encoding_1337_with_5_bit_prefix(self):
        val = decode_integer(b'\x1f\x9a\x0a', 5)
        assert val == (1337, 3)

    def test_encoding_42_with_8_bit_prefix(self):
        val = decode_integer(b'\x2a', 8)
        assert val == (42, 1)

    def test_decode_empty_string_fails(self):
        with pytest.raises(HPACKDecodingError):
            decode_integer(b'', 8)

    def test_decode_insufficient_data_fails(self):
        with pytest.raises(HPACKDecodingError):
            decode_integer(b'\x1f', 5)


class TestEncodingProperties:
    """
    Property-based tests for our integer encoder and decoder.
    """
    @given(
        integer=integers(min_value=0),
        prefix_bits=integers(min_value=1, max_value=8)
    )
    def test_encode_positive_integer_always_valid(self, integer, prefix_bits):
        """
        So long as the prefix bits are between 1 and 8, any positive integer
        can be represented.
        """
        result = encode_integer(integer, prefix_bits)
        assert isinstance(result, bytearray)
        assert len(result) > 0

    @given(
        integer=integers(max_value=-1),
        prefix_bits=integers(min_value=1, max_value=8)
    )
    def test_encode_fails_for_negative_integers(self, integer, prefix_bits):
        """
        If the integer to encode is negative, the encoder fails.
        """
        with pytest.raises(ValueError):
            encode_integer(integer, prefix_bits)

    @given(
        integer=integers(min_value=0),
        prefix_bits=one_of(
            integers(max_value=0),
            integers(min_value=9)
        )
    )
    def test_encode_fails_for_invalid_prefixes(self, integer, prefix_bits):
        """
        If the prefix is out of the range [1,8], the encoder fails.
        """
        with pytest.raises(ValueError):
            encode_integer(integer, prefix_bits)

    @given(
        prefix_bits=one_of(
            integers(max_value=0),
            integers(min_value=9)
        )
    )
    def test_decode_fails_for_invalid_prefixes(self, prefix_bits):
        """
        If the prefix is out of the range [1,8], the encoder fails.
        """
        with pytest.raises(ValueError):
            decode_integer(b'\x00', prefix_bits)

    @given(
        data=binary(),
        prefix_bits=integers(min_value=1, max_value=8)
    )
    def test_decode_either_succeeds_or_raises_error(self, data, prefix_bits):
        """
        Attempting to decode data either returns a positive integer or throws a
        HPACKDecodingError.
        """
        try:
            result, consumed = decode_integer(data, prefix_bits)
        except HPACKDecodingError:
            pass
        else:
            assert isinstance(result, int)
            assert result >= 0
            assert consumed > 0

    @given(
        integer=integers(min_value=0),
        prefix_bits=integers(min_value=1, max_value=8)
    )
    def test_encode_decode_round_trips(self, integer, prefix_bits):
        """
        Given valid data, the encoder and decoder can round trip.
        """
        encoded_result = encode_integer(integer, prefix_bits)
        decoded_integer, consumed = decode_integer(
            bytes(encoded_result), prefix_bits
        )
        assert integer == decoded_integer
        assert consumed > 0
