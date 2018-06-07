from __future__ import absolute_import, division, unicode_literals

from . import support  # noqa

import codecs
import sys
from io import BytesIO, StringIO

import pytest

import six
from six.moves import http_client, urllib

from html5lib._inputstream import (BufferedStream, HTMLInputStream,
                                   HTMLUnicodeInputStream, HTMLBinaryInputStream)
from html5lib._utils import supports_lone_surrogates


def test_basic():
    s = b"abc"
    fp = BufferedStream(BytesIO(s))
    read = fp.read(10)
    assert read == s


def test_read_length():
    fp = BufferedStream(BytesIO(b"abcdef"))
    read1 = fp.read(1)
    assert read1 == b"a"
    read2 = fp.read(2)
    assert read2 == b"bc"
    read3 = fp.read(3)
    assert read3 == b"def"
    read4 = fp.read(4)
    assert read4 == b""


def test_tell():
    fp = BufferedStream(BytesIO(b"abcdef"))
    read1 = fp.read(1)
    assert read1 == b"a"
    assert fp.tell() == 1
    read2 = fp.read(2)
    assert read2 == b"bc"
    assert fp.tell() == 3
    read3 = fp.read(3)
    assert read3 == b"def"
    assert fp.tell() == 6
    read4 = fp.read(4)
    assert read4 == b""
    assert fp.tell() == 6


def test_seek():
    fp = BufferedStream(BytesIO(b"abcdef"))
    read1 = fp.read(1)
    assert read1 == b"a"
    fp.seek(0)
    read2 = fp.read(1)
    assert read2 == b"a"
    read3 = fp.read(2)
    assert read3 == b"bc"
    fp.seek(2)
    read4 = fp.read(2)
    assert read4 == b"cd"
    fp.seek(4)
    read5 = fp.read(2)
    assert read5 == b"ef"


def test_seek_tell():
    fp = BufferedStream(BytesIO(b"abcdef"))
    read1 = fp.read(1)
    assert read1 == b"a"
    assert fp.tell() == 1
    fp.seek(0)
    read2 = fp.read(1)
    assert read2 == b"a"
    assert fp.tell() == 1
    read3 = fp.read(2)
    assert read3 == b"bc"
    assert fp.tell() == 3
    fp.seek(2)
    read4 = fp.read(2)
    assert read4 == b"cd"
    assert fp.tell() == 4
    fp.seek(4)
    read5 = fp.read(2)
    assert read5 == b"ef"
    assert fp.tell() == 6


class HTMLUnicodeInputStreamShortChunk(HTMLUnicodeInputStream):
    _defaultChunkSize = 2


class HTMLBinaryInputStreamShortChunk(HTMLBinaryInputStream):
    _defaultChunkSize = 2


def test_char_ascii():
    stream = HTMLInputStream(b"'", override_encoding='ascii')
    assert stream.charEncoding[0].name == 'windows-1252'
    assert stream.char() == "'"


def test_char_utf8():
    stream = HTMLInputStream('\u2018'.encode('utf-8'), override_encoding='utf-8')
    assert stream.charEncoding[0].name == 'utf-8'
    assert stream.char() == '\u2018'


def test_char_win1252():
    stream = HTMLInputStream("\xa9\xf1\u2019".encode('windows-1252'))
    assert stream.charEncoding[0].name == 'windows-1252'
    assert stream.char() == "\xa9"
    assert stream.char() == "\xf1"
    assert stream.char() == "\u2019"


def test_bom():
    stream = HTMLInputStream(codecs.BOM_UTF8 + b"'")
    assert stream.charEncoding[0].name == 'utf-8'
    assert stream.char() == "'"


def test_utf_16():
    stream = HTMLInputStream((' ' * 1025).encode('utf-16'))
    assert stream.charEncoding[0].name in ['utf-16le', 'utf-16be']
    assert len(stream.charsUntil(' ', True)) == 1025


def test_newlines():
    stream = HTMLBinaryInputStreamShortChunk(codecs.BOM_UTF8 + b"a\nbb\r\nccc\rddddxe")
    assert stream.position() == (1, 0)
    assert stream.charsUntil('c') == "a\nbb\n"
    assert stream.position() == (3, 0)
    assert stream.charsUntil('x') == "ccc\ndddd"
    assert stream.position() == (4, 4)
    assert stream.charsUntil('e') == "x"
    assert stream.position() == (4, 5)


def test_newlines2():
    size = HTMLUnicodeInputStream._defaultChunkSize
    stream = HTMLInputStream("\r" * size + "\n")
    assert stream.charsUntil('x') == "\n" * size


def test_position():
    stream = HTMLBinaryInputStreamShortChunk(codecs.BOM_UTF8 + b"a\nbb\nccc\nddde\nf\ngh")
    assert stream.position() == (1, 0)
    assert stream.charsUntil('c') == "a\nbb\n"
    assert stream.position() == (3, 0)
    stream.unget("\n")
    assert stream.position() == (2, 2)
    assert stream.charsUntil('c') == "\n"
    assert stream.position() == (3, 0)
    stream.unget("\n")
    assert stream.position() == (2, 2)
    assert stream.char() == "\n"
    assert stream.position() == (3, 0)
    assert stream.charsUntil('e') == "ccc\nddd"
    assert stream.position() == (4, 3)
    assert stream.charsUntil('h') == "e\nf\ng"
    assert stream.position() == (6, 1)


def test_position2():
    stream = HTMLUnicodeInputStreamShortChunk("abc\nd")
    assert stream.position() == (1, 0)
    assert stream.char() == "a"
    assert stream.position() == (1, 1)
    assert stream.char() == "b"
    assert stream.position() == (1, 2)
    assert stream.char() == "c"
    assert stream.position() == (1, 3)
    assert stream.char() == "\n"
    assert stream.position() == (2, 0)
    assert stream.char() == "d"
    assert stream.position() == (2, 1)


def test_python_issue_20007():
    """
    Make sure we have a work-around for Python bug #20007
    http://bugs.python.org/issue20007
    """
    class FakeSocket(object):
        def makefile(self, _mode, _bufsize=None):
            # pylint:disable=unused-argument
            return BytesIO(b"HTTP/1.1 200 Ok\r\n\r\nText")

    source = http_client.HTTPResponse(FakeSocket())
    source.begin()
    stream = HTMLInputStream(source)
    assert stream.charsUntil(" ") == "Text"


def test_python_issue_20007_b():
    """
    Make sure we have a work-around for Python bug #20007
    http://bugs.python.org/issue20007
    """
    if six.PY2:
        return

    class FakeSocket(object):
        def makefile(self, _mode, _bufsize=None):
            # pylint:disable=unused-argument
            return BytesIO(b"HTTP/1.1 200 Ok\r\n\r\nText")

    source = http_client.HTTPResponse(FakeSocket())
    source.begin()
    wrapped = urllib.response.addinfourl(source, source.msg, "http://example.com")
    stream = HTMLInputStream(wrapped)
    assert stream.charsUntil(" ") == "Text"


@pytest.mark.parametrize("inp,num",
                         [("\u0000", 0),
                          ("\u0001", 1),
                          ("\u0008", 1),
                          ("\u0009", 0),
                          ("\u000A", 0),
                          ("\u000B", 1),
                          ("\u000C", 0),
                          ("\u000D", 0),
                          ("\u000E", 1),
                          ("\u001F", 1),
                          ("\u0020", 0),
                          ("\u007E", 0),
                          ("\u007F", 1),
                          ("\u009F", 1),
                          ("\u00A0", 0),
                          ("\uFDCF", 0),
                          ("\uFDD0", 1),
                          ("\uFDEF", 1),
                          ("\uFDF0", 0),
                          ("\uFFFD", 0),
                          ("\uFFFE", 1),
                          ("\uFFFF", 1),
                          ("\U0001FFFD", 0),
                          ("\U0001FFFE", 1),
                          ("\U0001FFFF", 1),
                          ("\U0002FFFD", 0),
                          ("\U0002FFFE", 1),
                          ("\U0002FFFF", 1),
                          ("\U0003FFFD", 0),
                          ("\U0003FFFE", 1),
                          ("\U0003FFFF", 1),
                          ("\U0004FFFD", 0),
                          ("\U0004FFFE", 1),
                          ("\U0004FFFF", 1),
                          ("\U0005FFFD", 0),
                          ("\U0005FFFE", 1),
                          ("\U0005FFFF", 1),
                          ("\U0006FFFD", 0),
                          ("\U0006FFFE", 1),
                          ("\U0006FFFF", 1),
                          ("\U0007FFFD", 0),
                          ("\U0007FFFE", 1),
                          ("\U0007FFFF", 1),
                          ("\U0008FFFD", 0),
                          ("\U0008FFFE", 1),
                          ("\U0008FFFF", 1),
                          ("\U0009FFFD", 0),
                          ("\U0009FFFE", 1),
                          ("\U0009FFFF", 1),
                          ("\U000AFFFD", 0),
                          ("\U000AFFFE", 1),
                          ("\U000AFFFF", 1),
                          ("\U000BFFFD", 0),
                          ("\U000BFFFE", 1),
                          ("\U000BFFFF", 1),
                          ("\U000CFFFD", 0),
                          ("\U000CFFFE", 1),
                          ("\U000CFFFF", 1),
                          ("\U000DFFFD", 0),
                          ("\U000DFFFE", 1),
                          ("\U000DFFFF", 1),
                          ("\U000EFFFD", 0),
                          ("\U000EFFFE", 1),
                          ("\U000EFFFF", 1),
                          ("\U000FFFFD", 0),
                          ("\U000FFFFE", 1),
                          ("\U000FFFFF", 1),
                          ("\U0010FFFD", 0),
                          ("\U0010FFFE", 1),
                          ("\U0010FFFF", 1),
                          ("\x01\x01\x01", 3),
                          ("a\x01a\x01a\x01a", 3)])
def test_invalid_codepoints(inp, num):
    stream = HTMLUnicodeInputStream(StringIO(inp))
    for _i in range(len(inp)):
        stream.char()
    assert len(stream.errors) == num


@pytest.mark.skipif(not supports_lone_surrogates, reason="doesn't support lone surrogates")
@pytest.mark.parametrize("inp,num",
                         [("'\\uD7FF'", 0),
                          ("'\\uD800'", 1),
                          ("'\\uDBFF'", 1),
                          ("'\\uDC00'", 1),
                          ("'\\uDFFF'", 1),
                          ("'\\uE000'", 0),
                          ("'\\uD800\\uD800\\uD800'", 3),
                          ("'a\\uD800a\\uD800a\\uD800a'", 3),
                          ("'\\uDFFF\\uDBFF'", 2),
                          pytest.mark.skipif(sys.maxunicode == 0xFFFF,
                                             ("'\\uDBFF\\uDFFF'", 2),
                                             reason="narrow Python")])
def test_invalid_codepoints_surrogates(inp, num):
    inp = eval(inp)  # pylint:disable=eval-used
    fp = StringIO(inp)
    if ord(max(fp.read())) > 0xFFFF:
        pytest.skip("StringIO altered string")
    fp.seek(0)
    stream = HTMLUnicodeInputStream(fp)
    for _i in range(len(inp)):
        stream.char()
    assert len(stream.errors) == num
