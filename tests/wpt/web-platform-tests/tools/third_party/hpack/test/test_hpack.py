# -*- coding: utf-8 -*-
from hpack.hpack import Encoder, Decoder, _dict_to_iterable, _to_bytes
from hpack.exceptions import (
    HPACKDecodingError, InvalidTableIndex, OversizedHeaderListError,
    InvalidTableSizeError
)
from hpack.struct import HeaderTuple, NeverIndexedHeaderTuple
import itertools
import pytest

from hypothesis import given
from hypothesis.strategies import text, binary, sets, one_of

try:
    unicode = unicode
except NameError:
    unicode = str


class TestHPACKEncoder(object):
    # These tests are stolen entirely from the IETF specification examples.
    def test_literal_header_field_with_indexing(self):
        """
        The header field representation uses a literal name and a literal
        value.
        """
        e = Encoder()
        header_set = {'custom-key': 'custom-header'}
        result = b'\x40\x0acustom-key\x0dcustom-header'

        assert e.encode(header_set, huffman=False) == result
        assert list(e.header_table.dynamic_entries) == [
            (n.encode('utf-8'), v.encode('utf-8'))
            for n, v in header_set.items()
        ]

    def test_sensitive_headers(self):
        """
        Test encoding header values
        """
        e = Encoder()
        result = (b'\x82\x14\x88\x63\xa1\xa9' +
                  b'\x32\x08\x73\xd0\xc7\x10' +
                  b'\x87\x25\xa8\x49\xe9\xea' +
                  b'\x5f\x5f\x89\x41\x6a\x41' +
                  b'\x92\x6e\xe5\x35\x52\x9f')
        header_set = [
            (':method', 'GET', True),
            (':path', '/jimiscool/', True),
            ('customkey', 'sensitiveinfo', True),
        ]
        assert e.encode(header_set, huffman=True) == result

    def test_non_sensitive_headers_with_header_tuples(self):
        """
        A header field stored in a HeaderTuple emits a representation that
        allows indexing.
        """
        e = Encoder()
        result = (b'\x82\x44\x88\x63\xa1\xa9' +
                  b'\x32\x08\x73\xd0\xc7\x40' +
                  b'\x87\x25\xa8\x49\xe9\xea' +
                  b'\x5f\x5f\x89\x41\x6a\x41' +
                  b'\x92\x6e\xe5\x35\x52\x9f')
        header_set = [
            HeaderTuple(':method', 'GET'),
            HeaderTuple(':path', '/jimiscool/'),
            HeaderTuple('customkey', 'sensitiveinfo'),
        ]
        assert e.encode(header_set, huffman=True) == result

    def test_sensitive_headers_with_header_tuples(self):
        """
        A header field stored in a NeverIndexedHeaderTuple emits a
        representation that forbids indexing.
        """
        e = Encoder()
        result = (b'\x82\x14\x88\x63\xa1\xa9' +
                  b'\x32\x08\x73\xd0\xc7\x10' +
                  b'\x87\x25\xa8\x49\xe9\xea' +
                  b'\x5f\x5f\x89\x41\x6a\x41' +
                  b'\x92\x6e\xe5\x35\x52\x9f')
        header_set = [
            NeverIndexedHeaderTuple(':method', 'GET'),
            NeverIndexedHeaderTuple(':path', '/jimiscool/'),
            NeverIndexedHeaderTuple('customkey', 'sensitiveinfo'),
        ]
        assert e.encode(header_set, huffman=True) == result

    def test_header_table_size_getter(self):
        e = Encoder()
        assert e.header_table_size == 4096

    def test_indexed_literal_header_field_with_indexing(self):
        """
        The header field representation uses an indexed name and a literal
        value and performs incremental indexing.
        """
        e = Encoder()
        header_set = {':path': '/sample/path'}
        result = b'\x44\x0c/sample/path'

        assert e.encode(header_set, huffman=False) == result
        assert list(e.header_table.dynamic_entries) == [
            (n.encode('utf-8'), v.encode('utf-8'))
            for n, v in header_set.items()
        ]

    def test_indexed_header_field(self):
        """
        The header field representation uses an indexed header field, from
        the static table.
        """
        e = Encoder()
        header_set = {':method': 'GET'}
        result = b'\x82'

        assert e.encode(header_set, huffman=False) == result
        assert list(e.header_table.dynamic_entries) == []

    def test_indexed_header_field_from_static_table(self):
        e = Encoder()
        e.header_table_size = 0
        header_set = {':method': 'GET'}
        result = b'\x82'

        # Make sure we don't emit an encoding context update.
        e.header_table.resized = False

        assert e.encode(header_set, huffman=False) == result
        assert list(e.header_table.dynamic_entries) == []

    def test_request_examples_without_huffman(self):
        """
        This section shows several consecutive header sets, corresponding to
        HTTP requests, on the same connection.
        """
        e = Encoder()
        first_header_set = [
            (':method', 'GET',),
            (':scheme', 'http',),
            (':path', '/',),
            (':authority', 'www.example.com'),
        ]
        # We should have :authority in first_header_table since we index it
        first_header_table = [(':authority', 'www.example.com')]
        first_result = b'\x82\x86\x84\x41\x0fwww.example.com'

        assert e.encode(first_header_set, huffman=False) == first_result
        assert list(e.header_table.dynamic_entries) == [
            (n.encode('utf-8'), v.encode('utf-8'))
            for n, v in first_header_table
        ]

        second_header_set = [
            (':method', 'GET',),
            (':scheme', 'http',),
            (':path', '/',),
            (':authority', 'www.example.com',),
            ('cache-control', 'no-cache'),
        ]
        second_header_table = [
            ('cache-control', 'no-cache'),
            (':authority', 'www.example.com')
        ]
        second_result = b'\x82\x86\x84\xbeX\x08no-cache'

        assert e.encode(second_header_set, huffman=False) == second_result
        assert list(e.header_table.dynamic_entries) == [
            (n.encode('utf-8'), v.encode('utf-8'))
            for n, v in second_header_table
        ]

        third_header_set = [
            (':method', 'GET',),
            (':scheme', 'https',),
            (':path', '/index.html',),
            (':authority', 'www.example.com',),
            ('custom-key', 'custom-value'),
        ]
        third_result = (
            b'\x82\x87\x85\xbf@\ncustom-key\x0ccustom-value'
        )

        assert e.encode(third_header_set, huffman=False) == third_result
        # Don't check the header table here, it's just too complex to be
        # reliable. Check its length though.
        assert len(e.header_table.dynamic_entries) == 3

    def test_request_examples_with_huffman(self):
        """
        This section shows the same examples as the previous section, but
        using Huffman encoding for the literal values.
        """
        e = Encoder()
        first_header_set = [
            (':method', 'GET',),
            (':scheme', 'http',),
            (':path', '/',),
            (':authority', 'www.example.com'),
        ]
        first_header_table = [(':authority', 'www.example.com')]
        first_result = (
            b'\x82\x86\x84\x41\x8c\xf1\xe3\xc2\xe5\xf2:k\xa0\xab\x90\xf4\xff'
        )

        assert e.encode(first_header_set, huffman=True) == first_result
        assert list(e.header_table.dynamic_entries) == [
            (n.encode('utf-8'), v.encode('utf-8'))
            for n, v in first_header_table
        ]

        second_header_table = [
            ('cache-control', 'no-cache'),
            (':authority', 'www.example.com')
        ]
        second_header_set = [
            (':method', 'GET',),
            (':scheme', 'http',),
            (':path', '/',),
            (':authority', 'www.example.com',),
            ('cache-control', 'no-cache'),
        ]
        second_result = b'\x82\x86\x84\xbeX\x86\xa8\xeb\x10d\x9c\xbf'

        assert e.encode(second_header_set, huffman=True) == second_result
        assert list(e.header_table.dynamic_entries) == [
            (n.encode('utf-8'), v.encode('utf-8'))
            for n, v in second_header_table
        ]

        third_header_set = [
            (':method', 'GET',),
            (':scheme', 'https',),
            (':path', '/index.html',),
            (':authority', 'www.example.com',),
            ('custom-key', 'custom-value'),
        ]
        third_result = (
            b'\x82\x87\x85\xbf'
            b'@\x88%\xa8I\xe9[\xa9}\x7f\x89%\xa8I\xe9[\xb8\xe8\xb4\xbf'
        )

        assert e.encode(third_header_set, huffman=True) == third_result
        assert len(e.header_table.dynamic_entries) == 3

    # These tests are custom, for hyper.
    def test_resizing_header_table(self):
        # We need to encode a substantial number of headers, to populate the
        # header table.
        e = Encoder()
        header_set = [
            (':method', 'GET'),
            (':scheme', 'https'),
            (':path', '/some/path'),
            (':authority', 'www.example.com'),
            ('custom-key', 'custom-value'),
            (
                "user-agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.8; rv:16.0) "
                "Gecko/20100101 Firefox/16.0",
            ),
            (
                "accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;"
                "q=0.8",
            ),
            ('X-Lukasa-Test', '88989'),
        ]
        e.encode(header_set, huffman=True)

        # Resize the header table to a size so small that nothing can be in it.
        e.header_table_size = 40
        assert len(e.header_table.dynamic_entries) == 0

    def test_resizing_header_table_sends_multiple_updates(self):
        e = Encoder()

        e.header_table_size = 40
        e.header_table_size = 100
        e.header_table_size = 40

        header_set = [(':method', 'GET')]
        out = e.encode(header_set, huffman=True)
        assert out == b'\x3F\x09\x3F\x45\x3F\x09\x82'

    def test_resizing_header_table_to_same_size_ignored(self):
        e = Encoder()

        # These size changes should be ignored
        e.header_table_size = 4096
        e.header_table_size = 4096
        e.header_table_size = 4096

        # These size changes should be encoded
        e.header_table_size = 40
        e.header_table_size = 100
        e.header_table_size = 40

        header_set = [(':method', 'GET')]
        out = e.encode(header_set, huffman=True)
        assert out == b'\x3F\x09\x3F\x45\x3F\x09\x82'

    def test_resizing_header_table_sends_context_update(self):
        e = Encoder()

        # Resize the header table to a size so small that nothing can be in it.
        e.header_table_size = 40

        # Now, encode a header set. Just a small one, with a well-defined
        # output.
        header_set = [(':method', 'GET')]
        out = e.encode(header_set, huffman=True)

        assert out == b'?\t\x82'

    def test_setting_table_size_to_the_same_does_nothing(self):
        e = Encoder()

        # Set the header table size to the default.
        e.header_table_size = 4096

        # Now encode a header set. Just a small one, with a well-defined
        # output.
        header_set = [(':method', 'GET')]
        out = e.encode(header_set, huffman=True)

        assert out == b'\x82'

    def test_evicting_header_table_objects(self):
        e = Encoder()

        # Set the header table size large enough to include one header.
        e.header_table_size = 66
        header_set = [('a', 'b'), ('long-custom-header', 'longish value')]
        e.encode(header_set)

        assert len(e.header_table.dynamic_entries) == 1


class TestHPACKDecoder(object):
    # These tests are stolen entirely from the IETF specification examples.
    def test_literal_header_field_with_indexing(self):
        """
        The header field representation uses a literal name and a literal
        value.
        """
        d = Decoder()
        header_set = [('custom-key', 'custom-header')]
        data = b'\x40\x0acustom-key\x0dcustom-header'

        assert d.decode(data) == header_set
        assert list(d.header_table.dynamic_entries) == [
            (n.encode('utf-8'), v.encode('utf-8')) for n, v in header_set
        ]

    def test_raw_decoding(self):
        """
        The header field representation is decoded as a raw byte string instead
        of UTF-8
        """
        d = Decoder()
        header_set = [
            (b'\x00\x01\x99\x30\x11\x22\x55\x21\x89\x14', b'custom-header')
        ]
        data = (
            b'\x40\x0a\x00\x01\x99\x30\x11\x22\x55\x21\x89\x14\x0d'
            b'custom-header'
        )

        assert d.decode(data, raw=True) == header_set

    def test_literal_header_field_without_indexing(self):
        """
        The header field representation uses an indexed name and a literal
        value.
        """
        d = Decoder()
        header_set = [(':path', '/sample/path')]
        data = b'\x04\x0c/sample/path'

        assert d.decode(data) == header_set
        assert list(d.header_table.dynamic_entries) == []

    def test_header_table_size_getter(self):
        d = Decoder()
        assert d.header_table_size

    def test_indexed_header_field(self):
        """
        The header field representation uses an indexed header field, from
        the static table.
        """
        d = Decoder()
        header_set = [(':method', 'GET')]
        data = b'\x82'

        assert d.decode(data) == header_set
        assert list(d.header_table.dynamic_entries) == []

    def test_request_examples_without_huffman(self):
        """
        This section shows several consecutive header sets, corresponding to
        HTTP requests, on the same connection.
        """
        d = Decoder()
        first_header_set = [
            (':method', 'GET',),
            (':scheme', 'http',),
            (':path', '/',),
            (':authority', 'www.example.com'),
        ]
        # The first_header_table doesn't contain 'authority'
        first_data = b'\x82\x86\x84\x01\x0fwww.example.com'

        assert d.decode(first_data) == first_header_set
        assert list(d.header_table.dynamic_entries) == []

        # This request takes advantage of the differential encoding of header
        # sets.
        second_header_set = [
            (':method', 'GET',),
            (':scheme', 'http',),
            (':path', '/',),
            (':authority', 'www.example.com',),
            ('cache-control', 'no-cache'),
        ]
        second_data = (
            b'\x82\x86\x84\x01\x0fwww.example.com\x0f\t\x08no-cache'
        )

        assert d.decode(second_data) == second_header_set
        assert list(d.header_table.dynamic_entries) == []

        third_header_set = [
            (':method', 'GET',),
            (':scheme', 'https',),
            (':path', '/index.html',),
            (':authority', 'www.example.com',),
            ('custom-key', 'custom-value'),
        ]
        third_data = (
            b'\x82\x87\x85\x01\x0fwww.example.com@\ncustom-key\x0ccustom-value'
        )

        assert d.decode(third_data) == third_header_set
        # Don't check the header table here, it's just too complex to be
        # reliable. Check its length though.
        assert len(d.header_table.dynamic_entries) == 1

    def test_request_examples_with_huffman(self):
        """
        This section shows the same examples as the previous section, but
        using Huffman encoding for the literal values.
        """
        d = Decoder()

        first_header_set = [
            (':method', 'GET',),
            (':scheme', 'http',),
            (':path', '/',),
            (':authority', 'www.example.com'),
        ]
        first_data = (
            b'\x82\x86\x84\x01\x8c\xf1\xe3\xc2\xe5\xf2:k\xa0\xab\x90\xf4\xff'
        )

        assert d.decode(first_data) == first_header_set
        assert list(d.header_table.dynamic_entries) == []

        second_header_set = [
            (':method', 'GET',),
            (':scheme', 'http',),
            (':path', '/',),
            (':authority', 'www.example.com',),
            ('cache-control', 'no-cache'),
        ]
        second_data = (
            b'\x82\x86\x84\x01\x8c\xf1\xe3\xc2\xe5\xf2:k\xa0\xab\x90\xf4\xff'
            b'\x0f\t\x86\xa8\xeb\x10d\x9c\xbf'
        )

        assert d.decode(second_data) == second_header_set
        assert list(d.header_table.dynamic_entries) == []

        third_header_set = [
            (':method', 'GET',),
            (':scheme', 'https',),
            (':path', '/index.html',),
            (':authority', 'www.example.com',),
            ('custom-key', 'custom-value'),
        ]
        third_data = (
            b'\x82\x87\x85\x01\x8c\xf1\xe3\xc2\xe5\xf2:k\xa0\xab\x90\xf4\xff@'
            b'\x88%\xa8I\xe9[\xa9}\x7f\x89%\xa8I\xe9[\xb8\xe8\xb4\xbf'
        )

        assert d.decode(third_data) == third_header_set
        assert len(d.header_table.dynamic_entries) == 1

    # These tests are custom, for hyper.
    def test_resizing_header_table(self):
        # We need to decode a substantial number of headers, to populate the
        # header table. This string isn't magic: it's the output from the
        # equivalent test for the Encoder.
        d = Decoder()
        data = (
            b'\x82\x87D\x87a\x07\xa4\xacV4\xcfA\x8c\xf1\xe3\xc2\xe5\xf2:k\xa0'
            b'\xab\x90\xf4\xff@\x88%\xa8I\xe9[\xa9}\x7f\x89%\xa8I\xe9[\xb8\xe8'
            b'\xb4\xbfz\xbc\xd0\x7ff\xa2\x81\xb0\xda\xe0S\xfa\xd02\x1a\xa4\x9d'
            b'\x13\xfd\xa9\x92\xa4\x96\x854\x0c\x8aj\xdc\xa7\xe2\x81\x02\xef}'
            b'\xa9g{\x81qp\x7fjb):\x9d\x81\x00 \x00@\x150\x9a\xc2\xca\x7f,\x05'
            b'\xc5\xc1S\xb0I|\xa5\x89\xd3M\x1fC\xae\xba\x0cA\xa4\xc7\xa9\x8f3'
            b'\xa6\x9a?\xdf\x9ah\xfa\x1du\xd0b\r&=Ly\xa6\x8f\xbe\xd0\x01w\xfe'
            b'\xbeX\xf9\xfb\xed\x00\x17{@\x8a\xfc[=\xbdF\x81\xad\xbc\xa8O\x84y'
            b'\xe7\xde\x7f'
        )
        d.decode(data)

        # Resize the header table to a size so small that nothing can be in it.
        d.header_table_size = 40
        assert len(d.header_table.dynamic_entries) == 0

    def test_apache_trafficserver(self):
        # This test reproduces the bug in #110, using exactly the same header
        # data.
        d = Decoder()
        data = (
            b'\x10\x07:status\x03200@\x06server\tATS/6.0.0'
            b'@\x04date\x1dTue, 31 Mar 2015 08:09:51 GMT'
            b'@\x0ccontent-type\ttext/html@\x0econtent-length\x0542468'
            b'@\rlast-modified\x1dTue, 31 Mar 2015 01:55:51 GMT'
            b'@\x04vary\x0fAccept-Encoding@\x04etag\x0f"5519fea7-a5e4"'
            b'@\x08x-served\x05Nginx@\x14x-subdomain-tryfiles\x04True'
            b'@\x07x-deity\thydra-lts@\raccept-ranges\x05bytes@\x03age\x010'
            b'@\x19strict-transport-security\rmax-age=86400'
            b'@\x03via2https/1.1 ATS (ApacheTrafficServer/6.0.0 [cSsNfU])'
        )
        expect = [
            (':status', '200'),
            ('server', 'ATS/6.0.0'),
            ('date', 'Tue, 31 Mar 2015 08:09:51 GMT'),
            ('content-type', 'text/html'),
            ('content-length', '42468'),
            ('last-modified', 'Tue, 31 Mar 2015 01:55:51 GMT'),
            ('vary', 'Accept-Encoding'),
            ('etag', '"5519fea7-a5e4"'),
            ('x-served', 'Nginx'),
            ('x-subdomain-tryfiles', 'True'),
            ('x-deity', 'hydra-lts'),
            ('accept-ranges', 'bytes'),
            ('age', '0'),
            ('strict-transport-security', 'max-age=86400'),
            ('via', 'https/1.1 ATS (ApacheTrafficServer/6.0.0 [cSsNfU])'),
        ]

        result = d.decode(data)

        assert result == expect
        # The status header shouldn't be indexed.
        assert len(d.header_table.dynamic_entries) == len(expect) - 1

    def test_utf8_errors_raise_hpack_decoding_error(self):
        d = Decoder()

        # Invalid UTF-8 data.
        data = b'\x82\x86\x84\x01\x10www.\x07\xaa\xd7\x95\xd7\xa8\xd7\x94.com'

        with pytest.raises(HPACKDecodingError):
            d.decode(data)

    def test_invalid_indexed_literal(self):
        d = Decoder()

        # Refer to an index that is too large.
        data = b'\x82\x86\x84\x7f\x0a\x0fwww.example.com'
        with pytest.raises(InvalidTableIndex):
            d.decode(data)

    def test_invalid_indexed_header(self):
        d = Decoder()

        # Refer to an indexed header that is too large.
        data = b'\xBE\x86\x84\x01\x0fwww.example.com'
        with pytest.raises(InvalidTableIndex):
            d.decode(data)

    def test_literal_header_field_with_indexing_emits_headertuple(self):
        """
        A header field with indexing emits a HeaderTuple.
        """
        d = Decoder()
        data = b'\x00\x0acustom-key\x0dcustom-header'

        headers = d.decode(data)
        assert len(headers) == 1

        header = headers[0]
        assert isinstance(header, HeaderTuple)
        assert not isinstance(header, NeverIndexedHeaderTuple)

    def test_literal_never_indexed_emits_neverindexedheadertuple(self):
        """
        A literal header field that must never be indexed emits a
        NeverIndexedHeaderTuple.
        """
        d = Decoder()
        data = b'\x10\x0acustom-key\x0dcustom-header'

        headers = d.decode(data)
        assert len(headers) == 1

        header = headers[0]
        assert isinstance(header, NeverIndexedHeaderTuple)

    def test_indexed_never_indexed_emits_neverindexedheadertuple(self):
        """
        A header field with an indexed name that must never be indexed emits a
        NeverIndexedHeaderTuple.
        """
        d = Decoder()
        data = b'\x14\x0c/sample/path'

        headers = d.decode(data)
        assert len(headers) == 1

        header = headers[0]
        assert isinstance(header, NeverIndexedHeaderTuple)

    def test_max_header_list_size(self):
        """
        If the header block is larger than the max_header_list_size, the HPACK
        decoder throws an OversizedHeaderListError.
        """
        d = Decoder(max_header_list_size=44)
        data = b'\x14\x0c/sample/path'

        with pytest.raises(OversizedHeaderListError):
            d.decode(data)

    def test_can_decode_multiple_header_table_size_changes(self):
        """
        If multiple header table size changes are sent in at once, they are
        successfully decoded.
        """
        d = Decoder()
        data = b'?a?\xe1\x1f\x82\x87\x84A\x8a\x08\x9d\\\x0b\x81p\xdcy\xa6\x99'
        expect = [
            (':method', 'GET'),
            (':scheme', 'https'),
            (':path', '/'),
            (':authority', '127.0.0.1:8443')
        ]

        assert d.decode(data) == expect

    def test_header_table_size_change_above_maximum(self):
        """
        If a header table size change is received that exceeds the maximum
        allowed table size, it is rejected.
        """
        d = Decoder()
        d.max_allowed_table_size = 127
        data = b'?a\x82\x87\x84A\x8a\x08\x9d\\\x0b\x81p\xdcy\xa6\x99'

        with pytest.raises(InvalidTableSizeError):
            d.decode(data)

    def test_table_size_not_adjusting(self):
        """
        If the header table size is shrunk, and then the remote peer doesn't
        join in the shrinking, then an error is raised.
        """
        d = Decoder()
        d.max_allowed_table_size = 128
        data = b'\x82\x87\x84A\x8a\x08\x9d\\\x0b\x81p\xdcy\xa6\x99'

        with pytest.raises(InvalidTableSizeError):
            d.decode(data)

    def test_table_size_last_rejected(self):
        """
        If a header table size change comes last in the header block, it is
        forbidden.
        """
        d = Decoder()
        data = b'\x82\x87\x84A\x8a\x08\x9d\\\x0b\x81p\xdcy\xa6\x99?a'

        with pytest.raises(HPACKDecodingError):
            d.decode(data)

    def test_table_size_middle_rejected(self):
        """
        If a header table size change comes anywhere but first in the header
        block, it is forbidden.
        """
        d = Decoder()
        data = b'\x82?a\x87\x84A\x8a\x08\x9d\\\x0b\x81p\xdcy\xa6\x99'

        with pytest.raises(HPACKDecodingError):
            d.decode(data)

    def test_truncated_header_name(self):
        """
        If a header name is truncated an error is raised.
        """
        d = Decoder()
        # This is a simple header block that has a bad ending. The interesting
        # part begins on the second line. This indicates a string that has
        # literal name and value. The name is a 5 character huffman-encoded
        # string that is only three bytes long.
        data = (
            b'\x82\x87\x84A\x8a\x08\x9d\\\x0b\x81p\xdcy\xa6\x99'
            b'\x00\x85\xf2\xb2J'
        )

        with pytest.raises(HPACKDecodingError):
            d.decode(data)

    def test_truncated_header_value(self):
        """
        If a header value is truncated an error is raised.
        """
        d = Decoder()
        # This is a simple header block that has a bad ending. The interesting
        # part begins on the second line. This indicates a string that has
        # literal name and value. The name is a 5 character huffman-encoded
        # string, but the entire EOS character has been written over the end.
        # This causes hpack to see the header value as being supposed to be
        # 622462 bytes long, which it clearly is not, and so this must fail.
        data = (
            b'\x82\x87\x84A\x8a\x08\x9d\\\x0b\x81p\xdcy\xa6\x99'
            b'\x00\x85\xf2\xb2J\x87\xff\xff\xff\xfd%B\x7f'
        )

        with pytest.raises(HPACKDecodingError):
            d.decode(data)


class TestDictToIterable(object):
    """
    The dict_to_iterable function has some subtle requirements: validates that
    everything behaves as expected.

    As much as possible this tries to be exhaustive.
    """
    keys = one_of(
        text().filter(lambda k: k and not k.startswith(u':')),
        binary().filter(lambda k: k and not k.startswith(b':'))
    )

    @given(
        special_keys=sets(keys),
        boring_keys=sets(keys),
    )
    def test_ordering(self, special_keys, boring_keys):
        """
        _dict_to_iterable produces an iterable where all the keys beginning
        with a colon are emitted first.
        """
        def _prepend_colon(k):
            if isinstance(k, unicode):
                return u':' + k
            else:
                return b':' + k

        special_keys = set(map(_prepend_colon, special_keys))
        input_dict = {
            k: b'testval' for k in itertools.chain(
                special_keys,
                boring_keys
            )
        }
        filtered = _dict_to_iterable(input_dict)

        received_special = set()
        received_boring = set()

        for _ in special_keys:
            k, _ = next(filtered)
            received_special.add(k)
        for _ in boring_keys:
            k, _ = next(filtered)
            received_boring.add(k)

        assert special_keys == received_special
        assert boring_keys == received_boring

    @given(
        special_keys=sets(keys),
        boring_keys=sets(keys),
    )
    def test_ordering_applies_to_encoding(self, special_keys, boring_keys):
        """
        When encoding a dictionary the special keys all appear first.
        """
        def _prepend_colon(k):
            if isinstance(k, unicode):
                return u':' + k
            else:
                return b':' + k

        special_keys = set(map(_prepend_colon, special_keys))
        input_dict = {
            k: b'testval' for k in itertools.chain(
                special_keys,
                boring_keys
            )
        }
        e = Encoder()
        d = Decoder()
        encoded = e.encode(input_dict)
        decoded = iter(d.decode(encoded, raw=True))

        received_special = set()
        received_boring = set()
        expected_special = set(map(_to_bytes, special_keys))
        expected_boring = set(map(_to_bytes, boring_keys))

        for _ in special_keys:
            k, _ = next(decoded)
            received_special.add(k)
        for _ in boring_keys:
            k, _ = next(decoded)
            received_boring.add(k)

        assert expected_special == received_special
        assert expected_boring == received_boring
