#!/usr/bin/env python
#
# Copyright 2012, Google Inc.
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
"""Tests for handshake.base module."""

from __future__ import absolute_import

import unittest

import set_sys_path  # Update sys.path to locate pywebsocket3 module.
from pywebsocket3.common import (
    ExtensionParameter,
    ExtensionParsingException,
    format_extensions,
    parse_extensions,
)
from pywebsocket3.handshake.base import HandshakeException, validate_subprotocol


class ValidateSubprotocolTest(unittest.TestCase):
    """A unittest for validate_subprotocol method."""
    def test_validate_subprotocol(self):
        # Should succeed.
        validate_subprotocol('sample')
        validate_subprotocol('Sample')
        validate_subprotocol('sample\x7eprotocol')

        # Should fail.
        self.assertRaises(HandshakeException, validate_subprotocol, '')
        self.assertRaises(HandshakeException, validate_subprotocol,
                          'sample\x09protocol')
        self.assertRaises(HandshakeException, validate_subprotocol,
                          'sample\x19protocol')
        self.assertRaises(HandshakeException, validate_subprotocol,
                          'sample\x20protocol')
        self.assertRaises(HandshakeException, validate_subprotocol,
                          'sample\x7fprotocol')
        self.assertRaises(
            HandshakeException,
            validate_subprotocol,
            # "Japan" in Japanese
            u'\u65e5\u672c')


_TEST_TOKEN_EXTENSION_DATA = [
    ('foo', [('foo', [])]),
    ('foo; bar', [('foo', [('bar', None)])]),
    ('foo; bar=baz', [('foo', [('bar', 'baz')])]),
    ('foo; bar=baz; car=cdr', [('foo', [('bar', 'baz'), ('car', 'cdr')])]),
    ('foo; bar=baz, car; cdr', [('foo', [('bar', 'baz')]),
                                ('car', [('cdr', None)])]),
    ('a, b, c, d', [('a', []), ('b', []), ('c', []), ('d', [])]),
]

_TEST_QUOTED_EXTENSION_DATA = [
    ('foo; bar=""', [('foo', [('bar', '')])]),
    ('foo; bar=" baz "', [('foo', [('bar', ' baz ')])]),
    ('foo; bar=",baz;"', [('foo', [('bar', ',baz;')])]),
    ('foo; bar="\\\r\\\nbaz"', [('foo', [('bar', '\r\nbaz')])]),
    ('foo; bar="\\"baz"', [('foo', [('bar', '"baz')])]),
    ('foo; bar="\xbbbaz"', [('foo', [('bar', '\xbbbaz')])]),
]

_TEST_REDUNDANT_TOKEN_EXTENSION_DATA = [
    ('foo \t ', [('foo', [])]),
    ('foo; \r\n bar', [('foo', [('bar', None)])]),
    ('foo; bar=\r\n \r\n baz', [('foo', [('bar', 'baz')])]),
    ('foo ;bar = baz ', [('foo', [('bar', 'baz')])]),
    ('foo,bar,,baz', [('foo', []), ('bar', []), ('baz', [])]),
]

_TEST_REDUNDANT_QUOTED_EXTENSION_DATA = [
    ('foo; bar="\r\n \r\n baz"', [('foo', [('bar', '  baz')])]),
]


class ExtensionsParserTest(unittest.TestCase):
    def _verify_extension_list(self, expected_list, actual_list):
        """Verifies that ExtensionParameter objects in actual_list have the
        same members as extension definitions in expected_list. Extension
        definition used in this test is a pair of an extension name and a
        parameter dictionary.
        """

        self.assertEqual(len(expected_list), len(actual_list))
        for expected, actual in zip(expected_list, actual_list):
            (name, parameters) = expected
            self.assertEqual(name, actual._name)
            self.assertEqual(parameters, actual._parameters)

    def test_parse(self):
        for formatted_string, definition in _TEST_TOKEN_EXTENSION_DATA:
            self._verify_extension_list(definition,
                                        parse_extensions(formatted_string))

    def test_parse_quoted_data(self):
        for formatted_string, definition in _TEST_QUOTED_EXTENSION_DATA:
            self._verify_extension_list(definition,
                                        parse_extensions(formatted_string))

    def test_parse_redundant_data(self):
        for (formatted_string,
             definition) in _TEST_REDUNDANT_TOKEN_EXTENSION_DATA:
            self._verify_extension_list(definition,
                                        parse_extensions(formatted_string))

    def test_parse_redundant_quoted_data(self):
        for (formatted_string,
             definition) in _TEST_REDUNDANT_QUOTED_EXTENSION_DATA:
            self._verify_extension_list(definition,
                                        parse_extensions(formatted_string))

    def test_parse_bad_data(self):
        _TEST_BAD_EXTENSION_DATA = [
            ('foo; ; '),
            ('foo; a a'),
            ('foo foo'),
            (',,,'),
            ('foo; bar='),
            ('foo; bar="hoge'),
            ('foo; bar="a\r"'),
            ('foo; bar="\\\xff"'),
            ('foo; bar=\ra'),
        ]

        for formatted_string in _TEST_BAD_EXTENSION_DATA:
            self.assertRaises(ExtensionParsingException, parse_extensions,
                              formatted_string)


class FormatExtensionsTest(unittest.TestCase):
    def test_format_extensions(self):
        for formatted_string, definitions in _TEST_TOKEN_EXTENSION_DATA:
            extensions = []
            for definition in definitions:
                (name, parameters) = definition
                extension = ExtensionParameter(name)
                extension._parameters = parameters
                extensions.append(extension)
            self.assertEqual(formatted_string, format_extensions(extensions))


if __name__ == '__main__':
    unittest.main()

# vi:sts=4 sw=4 et
