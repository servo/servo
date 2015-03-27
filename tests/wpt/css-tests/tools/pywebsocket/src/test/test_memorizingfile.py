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


"""Tests for memorizingfile module."""


import StringIO
import unittest

import set_sys_path  # Update sys.path to locate mod_pywebsocket module.

from mod_pywebsocket import memorizingfile


class UtilTest(unittest.TestCase):
    """A unittest for memorizingfile module."""

    def check(self, memorizing_file, num_read, expected_list):
        for unused in range(num_read):
            memorizing_file.readline()
        actual_list = memorizing_file.get_memorized_lines()
        self.assertEqual(len(expected_list), len(actual_list))
        for expected, actual in zip(expected_list, actual_list):
            self.assertEqual(expected, actual)

    def check_with_size(self, memorizing_file, read_size, expected_list):
        read_list = []
        read_line = ''
        while True:
            line = memorizing_file.readline(read_size)
            line_length = len(line)
            self.assertTrue(line_length <= read_size)
            if line_length == 0:
                if read_line != '':
                    read_list.append(read_line)
                break
            read_line += line
            if line[line_length - 1] == '\n':
                read_list.append(read_line)
                read_line = ''
        actual_list = memorizing_file.get_memorized_lines()
        self.assertEqual(len(expected_list), len(actual_list))
        self.assertEqual(len(expected_list), len(read_list))
        for expected, actual, read in zip(expected_list, actual_list,
                                          read_list):
            self.assertEqual(expected, actual)
            self.assertEqual(expected, read)

    def test_get_memorized_lines(self):
        memorizing_file = memorizingfile.MemorizingFile(StringIO.StringIO(
                'Hello\nWorld\nWelcome'))
        self.check(memorizing_file, 3, ['Hello\n', 'World\n', 'Welcome'])

    def test_get_memorized_lines_limit_memorized_lines(self):
        memorizing_file = memorizingfile.MemorizingFile(StringIO.StringIO(
                'Hello\nWorld\nWelcome'), 2)
        self.check(memorizing_file, 3, ['Hello\n', 'World\n'])

    def test_get_memorized_lines_empty_file(self):
        memorizing_file = memorizingfile.MemorizingFile(StringIO.StringIO(
                ''))
        self.check(memorizing_file, 10, [])

    def test_get_memorized_lines_with_size(self):
        for size in range(1, 10):
            memorizing_file = memorizingfile.MemorizingFile(StringIO.StringIO(
                'Hello\nWorld\nWelcome'))
            self.check_with_size(memorizing_file, size,
                                 ['Hello\n', 'World\n', 'Welcome'])

if __name__ == '__main__':
    unittest.main()


# vi:sts=4 sw=4 et
