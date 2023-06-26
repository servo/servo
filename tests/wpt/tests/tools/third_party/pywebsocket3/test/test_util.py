#!/usr/bin/env python
#
# Copyright 2011, Google Inc.
# All rights reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are
# met:
#
#     * Redistributions of source code must retain the above copyright
# notice, this list of conditions and the following disclaimer.
#     * Redistributions in binary form must reproduce the above
# copyright notice, this list of conditions and the following disclaimer
# in the documentation and/or other materials provided with the
# distribution.
#     * Neither the name of Google Inc. nor the names of its
# contributors may be used to endorse or promote products derived from
# this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
# A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
# OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
# LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
# DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
# THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
# (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
"""Tests for util module."""

from __future__ import absolute_import
from __future__ import print_function
import os
import random
import sys
import unittest
import struct

import set_sys_path  # Update sys.path to locate mod_pywebsocket module.

from mod_pywebsocket import util
from six.moves import range
from six import PY3
from six import int2byte

_TEST_DATA_DIR = os.path.join(os.path.dirname(__file__), 'testdata')


class UtilTest(unittest.TestCase):
    """A unittest for util module."""
    def test_prepend_message_to_exception(self):
        exc = Exception('World')
        self.assertEqual('World', str(exc))
        util.prepend_message_to_exception('Hello ', exc)
        self.assertEqual('Hello World', str(exc))

    def test_get_script_interp(self):
        cygwin_path = 'c:\\cygwin\\bin'
        cygwin_perl = os.path.join(cygwin_path, 'perl')
        self.assertEqual(
            None, util.get_script_interp(os.path.join(_TEST_DATA_DIR,
                                                      'README')))
        self.assertEqual(
            None,
            util.get_script_interp(os.path.join(_TEST_DATA_DIR, 'README'),
                                   cygwin_path))
        self.assertEqual(
            '/usr/bin/perl -wT',
            util.get_script_interp(os.path.join(_TEST_DATA_DIR, 'hello.pl')))
        self.assertEqual(
            cygwin_perl + ' -wT',
            util.get_script_interp(os.path.join(_TEST_DATA_DIR, 'hello.pl'),
                                   cygwin_path))

    def test_hexify(self):
        self.assertEqual('61 7a 41 5a 30 39 20 09 0d 0a 00 ff',
                         util.hexify(b'azAZ09 \t\r\n\x00\xff'))


class RepeatedXorMaskerTest(unittest.TestCase):
    """A unittest for RepeatedXorMasker class."""
    def test_mask(self):
        # Sample input e6,97,a5 is U+65e5 in UTF-8
        masker = util.RepeatedXorMasker(b'\xff\xff\xff\xff')
        result = masker.mask(b'\xe6\x97\xa5')
        self.assertEqual(b'\x19\x68\x5a', result)

        masker = util.RepeatedXorMasker(b'\x00\x00\x00\x00')
        result = masker.mask(b'\xe6\x97\xa5')
        self.assertEqual(b'\xe6\x97\xa5', result)

        masker = util.RepeatedXorMasker(b'\xe6\x97\xa5\x20')
        result = masker.mask(b'\xe6\x97\xa5')
        self.assertEqual(b'\x00\x00\x00', result)

    def test_mask_twice(self):
        masker = util.RepeatedXorMasker(b'\x00\x7f\xff\x20')
        # mask[0], mask[1], ... will be used.
        result = masker.mask(b'\x00\x00\x00\x00\x00')
        self.assertEqual(b'\x00\x7f\xff\x20\x00', result)
        # mask[2], mask[0], ... will be used for the next call.
        result = masker.mask(b'\x00\x00\x00\x00\x00')
        self.assertEqual(b'\x7f\xff\x20\x00\x7f', result)

    def test_mask_large_data(self):
        masker = util.RepeatedXorMasker(b'mASk')
        original = b''.join([util.pack_byte(i % 256) for i in range(1000)])
        result = masker.mask(original)
        expected = b''.join([
            util.pack_byte((i % 256) ^ ord('mASk'[i % 4])) for i in range(1000)
        ])
        self.assertEqual(expected, result)

        masker = util.RepeatedXorMasker(b'MaSk')
        first_part = b'The WebSocket Protocol enables two-way communication.'
        result = masker.mask(first_part)
        self.assertEqual(
            b'\x19\t6K\x1a\x0418"\x028\x0e9A\x03\x19"\x15<\x08"\rs\x0e#'
            b'\x001\x07(\x12s\x1f:\x0e~\x1c,\x18s\x08"\x0c>\x1e#\x080\n9'
            b'\x08<\x05c', result)
        second_part = b'It has two parts: a handshake and the data transfer.'
        result = masker.mask(second_part)
        self.assertEqual(
            b"('K%\x00 K9\x16<K=\x00!\x1f>[s\nm\t2\x05)\x12;\n&\x04s\n#"
            b"\x05s\x1f%\x04s\x0f,\x152K9\x132\x05>\x076\x19c", result)


def get_random_section(source, min_num_chunks):
    chunks = []
    bytes_chunked = 0

    while bytes_chunked < len(source):
        chunk_size = random.randint(
            1, min(len(source) / min_num_chunks,
                   len(source) - bytes_chunked))
        chunk = source[bytes_chunked:bytes_chunked + chunk_size]
        chunks.append(chunk)
        bytes_chunked += chunk_size

    return chunks


class InflaterDeflaterTest(unittest.TestCase):
    """A unittest for _Inflater and _Deflater class."""
    def test_inflate_deflate_default(self):
        input = b'hello' + b'-' * 30000 + b'hello'
        inflater15 = util._Inflater(15)
        deflater15 = util._Deflater(15)
        inflater8 = util._Inflater(8)
        deflater8 = util._Deflater(8)

        compressed15 = deflater15.compress_and_finish(input)
        compressed8 = deflater8.compress_and_finish(input)

        inflater15.append(compressed15)
        inflater8.append(compressed8)

        self.assertNotEqual(compressed15, compressed8)
        self.assertEqual(input, inflater15.decompress(-1))
        self.assertEqual(input, inflater8.decompress(-1))

    def test_random_section(self):
        random.seed(a=0)
        source = b''.join(
            [int2byte(random.randint(0, 255)) for i in range(100 * 1024)])

        chunked_input = get_random_section(source, 10)

        deflater = util._Deflater(15)
        compressed = []
        for chunk in chunked_input:
            compressed.append(deflater.compress(chunk))
        compressed.append(deflater.compress_and_finish(b''))

        chunked_expectation = get_random_section(source, 10)

        inflater = util._Inflater(15)
        inflater.append(b''.join(compressed))
        for chunk in chunked_expectation:
            decompressed = inflater.decompress(len(chunk))
            self.assertEqual(chunk, decompressed)

        self.assertEqual(b'', inflater.decompress(-1))


if __name__ == '__main__':
    unittest.main()

# vi:sts=4 sw=4 et
