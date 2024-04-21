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
"""Tests for http_header_util module."""

from __future__ import absolute_import

import unittest
import sys

from pywebsocket3 import http_header_util


class UnitTest(unittest.TestCase):
    """A unittest for http_header_util module."""
    def test_parse_relative_uri(self):
        host, port, resource = http_header_util.parse_uri('/ws/test')
        self.assertEqual(None, host)
        self.assertEqual(None, port)
        self.assertEqual('/ws/test', resource)

    def test_parse_absolute_uri(self):
        host, port, resource = http_header_util.parse_uri(
            'ws://localhost:10080/ws/test')
        self.assertEqual('localhost', host)
        self.assertEqual(10080, port)
        self.assertEqual('/ws/test', resource)

        host, port, resource = http_header_util.parse_uri(
            'ws://example.com/ws/test')
        self.assertEqual('example.com', host)
        self.assertEqual(80, port)
        self.assertEqual('/ws/test', resource)

        host, port, resource = http_header_util.parse_uri('wss://example.com/')
        self.assertEqual('example.com', host)
        self.assertEqual(443, port)
        self.assertEqual('/', resource)

        host, port, resource = http_header_util.parse_uri(
            'ws://example.com:8080')
        self.assertEqual('example.com', host)
        self.assertEqual(8080, port)
        self.assertEqual('/', resource)

    def test_parse_invalid_uri(self):
        host, port, resource = http_header_util.parse_uri('ws:///')
        self.assertEqual(None, resource)

        host, port, resource = http_header_util.parse_uri(
            'ws://localhost:INVALID_PORT')
        self.assertEqual(None, resource)

        host, port, resource = http_header_util.parse_uri(
            'ws://localhost:-1/ws')
        if sys.hexversion >= 0x030600f0:
            self.assertEqual(None, resource)
        else:
            self.assertEqual('localhost', host)
            self.assertEqual(80, port)
            self.assertEqual('/ws', resource)


if __name__ == '__main__':
    unittest.main()

# vi:sts=4 sw=4 et
