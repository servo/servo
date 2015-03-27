from __future__ import absolute_import, division, unicode_literals

from . import support  # flake8: noqa
import unittest
import codecs
from io import BytesIO

from six.moves import http_client

from html5lib.inputstream import (BufferedStream, HTMLInputStream,
                                  HTMLUnicodeInputStream, HTMLBinaryInputStream)

class BufferedStreamTest(unittest.TestCase):
    def test_basic(self):
        s = b"abc"
        fp = BufferedStream(BytesIO(s))
        read = fp.read(10)
        assert read == s

    def test_read_length(self):
        fp = BufferedStream(BytesIO(b"abcdef"))
        read1 = fp.read(1)
        assert read1 == b"a"
        read2 = fp.read(2)
        assert read2 == b"bc"
        read3 = fp.read(3)
        assert read3 == b"def"
        read4 = fp.read(4)
        assert read4 == b""

    def test_tell(self):
        fp = BufferedStream(BytesIO(b"abcdef"))
        read1 = fp.read(1)
        assert fp.tell() == 1
        read2 = fp.read(2)
        assert fp.tell() == 3
        read3 = fp.read(3)
        assert fp.tell() == 6
        read4 = fp.read(4)
        assert fp.tell() == 6

    def test_seek(self):
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

    def test_seek_tell(self):
        fp = BufferedStream(BytesIO(b"abcdef"))
        read1 = fp.read(1)
        assert fp.tell() == 1
        fp.seek(0)
        read2 = fp.read(1)
        assert fp.tell() == 1
        read3 = fp.read(2)
        assert fp.tell() == 3
        fp.seek(2)
        read4 = fp.read(2)
        assert fp.tell() == 4
        fp.seek(4)
        read5 = fp.read(2)
        assert fp.tell() == 6


class HTMLUnicodeInputStreamShortChunk(HTMLUnicodeInputStream):
    _defaultChunkSize = 2


class HTMLBinaryInputStreamShortChunk(HTMLBinaryInputStream):
    _defaultChunkSize = 2


class HTMLInputStreamTest(unittest.TestCase):

    def test_char_ascii(self):
        stream = HTMLInputStream(b"'", encoding='ascii')
        self.assertEqual(stream.charEncoding[0], 'ascii')
        self.assertEqual(stream.char(), "'")

    def test_char_utf8(self):
        stream = HTMLInputStream('\u2018'.encode('utf-8'), encoding='utf-8')
        self.assertEqual(stream.charEncoding[0], 'utf-8')
        self.assertEqual(stream.char(), '\u2018')

    def test_char_win1252(self):
        stream = HTMLInputStream("\xa9\xf1\u2019".encode('windows-1252'))
        self.assertEqual(stream.charEncoding[0], 'windows-1252')
        self.assertEqual(stream.char(), "\xa9")
        self.assertEqual(stream.char(), "\xf1")
        self.assertEqual(stream.char(), "\u2019")

    def test_bom(self):
        stream = HTMLInputStream(codecs.BOM_UTF8 + b"'")
        self.assertEqual(stream.charEncoding[0], 'utf-8')
        self.assertEqual(stream.char(), "'")

    def test_utf_16(self):
        stream = HTMLInputStream((' ' * 1025).encode('utf-16'))
        self.assertTrue(stream.charEncoding[0] in ['utf-16-le', 'utf-16-be'], stream.charEncoding)
        self.assertEqual(len(stream.charsUntil(' ', True)), 1025)

    def test_newlines(self):
        stream = HTMLBinaryInputStreamShortChunk(codecs.BOM_UTF8 + b"a\nbb\r\nccc\rddddxe")
        self.assertEqual(stream.position(), (1, 0))
        self.assertEqual(stream.charsUntil('c'), "a\nbb\n")
        self.assertEqual(stream.position(), (3, 0))
        self.assertEqual(stream.charsUntil('x'), "ccc\ndddd")
        self.assertEqual(stream.position(), (4, 4))
        self.assertEqual(stream.charsUntil('e'), "x")
        self.assertEqual(stream.position(), (4, 5))

    def test_newlines2(self):
        size = HTMLUnicodeInputStream._defaultChunkSize
        stream = HTMLInputStream("\r" * size + "\n")
        self.assertEqual(stream.charsUntil('x'), "\n" * size)

    def test_position(self):
        stream = HTMLBinaryInputStreamShortChunk(codecs.BOM_UTF8 + b"a\nbb\nccc\nddde\nf\ngh")
        self.assertEqual(stream.position(), (1, 0))
        self.assertEqual(stream.charsUntil('c'), "a\nbb\n")
        self.assertEqual(stream.position(), (3, 0))
        stream.unget("\n")
        self.assertEqual(stream.position(), (2, 2))
        self.assertEqual(stream.charsUntil('c'), "\n")
        self.assertEqual(stream.position(), (3, 0))
        stream.unget("\n")
        self.assertEqual(stream.position(), (2, 2))
        self.assertEqual(stream.char(), "\n")
        self.assertEqual(stream.position(), (3, 0))
        self.assertEqual(stream.charsUntil('e'), "ccc\nddd")
        self.assertEqual(stream.position(), (4, 3))
        self.assertEqual(stream.charsUntil('h'), "e\nf\ng")
        self.assertEqual(stream.position(), (6, 1))

    def test_position2(self):
        stream = HTMLUnicodeInputStreamShortChunk("abc\nd")
        self.assertEqual(stream.position(), (1, 0))
        self.assertEqual(stream.char(), "a")
        self.assertEqual(stream.position(), (1, 1))
        self.assertEqual(stream.char(), "b")
        self.assertEqual(stream.position(), (1, 2))
        self.assertEqual(stream.char(), "c")
        self.assertEqual(stream.position(), (1, 3))
        self.assertEqual(stream.char(), "\n")
        self.assertEqual(stream.position(), (2, 0))
        self.assertEqual(stream.char(), "d")
        self.assertEqual(stream.position(), (2, 1))

    def test_python_issue_20007(self):
        """
        Make sure we have a work-around for Python bug #20007
        http://bugs.python.org/issue20007
        """
        class FakeSocket(object):
            def makefile(self, _mode, _bufsize=None):
                return BytesIO(b"HTTP/1.1 200 Ok\r\n\r\nText")

        source = http_client.HTTPResponse(FakeSocket())
        source.begin()
        stream = HTMLInputStream(source)
        self.assertEqual(stream.charsUntil(" "), "Text")


def buildTestSuite():
    return unittest.defaultTestLoader.loadTestsFromName(__name__)


def main():
    buildTestSuite()
    unittest.main()

if __name__ == '__main__':
    main()
