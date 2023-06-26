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
"""Tests for stream module."""

from __future__ import absolute_import
import unittest

import set_sys_path  # Update sys.path to locate mod_pywebsocket module.

from mod_pywebsocket import common
from mod_pywebsocket import stream


class StreamTest(unittest.TestCase):
    """A unittest for stream module."""
    def test_create_header(self):
        # more, rsv1, ..., rsv4 are all true
        header = stream.create_header(common.OPCODE_TEXT, 1, 1, 1, 1, 1, 1)
        self.assertEqual(b'\xf1\x81', header)

        # Maximum payload size
        header = stream.create_header(common.OPCODE_TEXT, (1 << 63) - 1, 0, 0,
                                      0, 0, 0)
        self.assertEqual(b'\x01\x7f\x7f\xff\xff\xff\xff\xff\xff\xff', header)

        # Invalid opcode 0x10
        self.assertRaises(ValueError, stream.create_header, 0x10, 0, 0, 0, 0,
                          0, 0)

        # Invalid value 0xf passed to more parameter
        self.assertRaises(ValueError, stream.create_header, common.OPCODE_TEXT,
                          0, 0xf, 0, 0, 0, 0)

        # Too long payload_length
        self.assertRaises(ValueError, stream.create_header, common.OPCODE_TEXT,
                          1 << 63, 0, 0, 0, 0, 0)


if __name__ == '__main__':
    unittest.main()

# vi:sts=4 sw=4 et
