# -*- coding: utf-8 -*-
from hpack.exceptions import HPACKDecodingError
from hpack.huffman_table import decode_huffman
from hpack.huffman import HuffmanEncoder
from hpack.huffman_constants import REQUEST_CODES, REQUEST_CODES_LENGTH

from hypothesis import given, example
from hypothesis.strategies import binary


class TestHuffman(object):

    def test_request_huffman_decoder(self):
        assert (
            decode_huffman(b'\xf1\xe3\xc2\xe5\xf2:k\xa0\xab\x90\xf4\xff') ==
            b"www.example.com"
        )
        assert decode_huffman(b'\xa8\xeb\x10d\x9c\xbf') == b"no-cache"
        assert decode_huffman(b'%\xa8I\xe9[\xa9}\x7f') == b"custom-key"
        assert (
            decode_huffman(b'%\xa8I\xe9[\xb8\xe8\xb4\xbf') == b"custom-value"
        )

    def test_request_huffman_encode(self):
        encoder = HuffmanEncoder(REQUEST_CODES, REQUEST_CODES_LENGTH)
        assert (
            encoder.encode(b"www.example.com") ==
            b'\xf1\xe3\xc2\xe5\xf2:k\xa0\xab\x90\xf4\xff'
        )
        assert encoder.encode(b"no-cache") == b'\xa8\xeb\x10d\x9c\xbf'
        assert encoder.encode(b"custom-key") == b'%\xa8I\xe9[\xa9}\x7f'
        assert (
            encoder.encode(b"custom-value") == b'%\xa8I\xe9[\xb8\xe8\xb4\xbf'
        )


class TestHuffmanDecoder(object):
    @given(data=binary())
    @example(b'\xff')
    @example(b'\x5f\xff\xff\xff\xff')
    @example(b'\x00\x3f\xff\xff\xff')
    def test_huffman_decoder_properly_handles_all_bytestrings(self, data):
        """
        When given random bytestrings, either we get HPACKDecodingError or we
        get a bytestring back.
        """
        # The examples aren't special, they're just known to hit specific error
        # paths through the state machine. Basically, they are strings that are
        # definitely invalid.
        try:
            result = decode_huffman(data)
        except HPACKDecodingError:
            result = b''

        assert isinstance(result, bytes)
