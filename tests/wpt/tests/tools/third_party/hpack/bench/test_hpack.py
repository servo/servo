from hpack.hpack import (
    encode_integer,
    decode_integer
)


class TestHpackEncodingIntegersBenchmarks:
    def test_encode_small_integer_large_prefix(self, benchmark):
        benchmark(encode_integer, integer=120, prefix_bits=7)

    def test_encode_small_integer_small_prefix(self, benchmark):
        benchmark(encode_integer, integer=120, prefix_bits=1)

    def test_encode_large_integer_large_prefix(self, benchmark):
        benchmark(encode_integer, integer=120000, prefix_bits=7)

    def test_encode_large_integer_small_prefix(self, benchmark):
        benchmark(encode_integer, integer=120000, prefix_bits=1)


class TestHpackDecodingIntegersBenchmarks:
    def test_decode_small_integer_large_prefix(self, benchmark):
        data = bytes(encode_integer(integer=120, prefix_bits=7))
        benchmark(decode_integer, data=data, prefix_bits=7)

    def test_decode_small_integer_small_prefix(self, benchmark):
        data = bytes(encode_integer(integer=120, prefix_bits=1))
        benchmark(decode_integer, data=data, prefix_bits=1)

    def test_decode_large_integer_large_prefix(self, benchmark):
        data = bytes(encode_integer(integer=120000, prefix_bits=7))
        benchmark(decode_integer, data=data, prefix_bits=7)

    def test_decode_large_integer_small_prefix(self, benchmark):
        data = bytes(encode_integer(integer=120000, prefix_bits=1))
        benchmark(decode_integer, data=data, prefix_bits=1)
