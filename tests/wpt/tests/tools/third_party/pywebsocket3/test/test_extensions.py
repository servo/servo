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
"""Tests for extensions module."""

from __future__ import absolute_import

import unittest
import zlib

import set_sys_path  # Update sys.path to locate pywebsocket3 module.
from pywebsocket3 import common, extensions


class ExtensionsTest(unittest.TestCase):
    """A unittest for non-class methods in extensions.py"""
    def test_parse_window_bits(self):
        self.assertRaises(ValueError, extensions._parse_window_bits, None)
        self.assertRaises(ValueError, extensions._parse_window_bits, 'foobar')
        self.assertRaises(ValueError, extensions._parse_window_bits, ' 8 ')
        self.assertRaises(ValueError, extensions._parse_window_bits, 'a8a')
        self.assertRaises(ValueError, extensions._parse_window_bits, '00000')
        self.assertRaises(ValueError, extensions._parse_window_bits, '00008')
        self.assertRaises(ValueError, extensions._parse_window_bits, '0x8')

        self.assertRaises(ValueError, extensions._parse_window_bits, '9.5')
        self.assertRaises(ValueError, extensions._parse_window_bits, '8.0')

        self.assertTrue(extensions._parse_window_bits, '8')
        self.assertTrue(extensions._parse_window_bits, '15')

        self.assertRaises(ValueError, extensions._parse_window_bits, '-8')
        self.assertRaises(ValueError, extensions._parse_window_bits, '0')
        self.assertRaises(ValueError, extensions._parse_window_bits, '7')

        self.assertRaises(ValueError, extensions._parse_window_bits, '16')
        self.assertRaises(ValueError, extensions._parse_window_bits,
                          '10000000')


class PerMessageDeflateExtensionProcessorParsingTest(unittest.TestCase):
    """A unittest for checking that PerMessageDeflateExtensionProcessor parses
    given extension parameter correctly.
    """
    def test_registry(self):
        processor = extensions.get_extension_processor(
            common.ExtensionParameter('permessage-deflate'))
        self.assertIsInstance(processor,
                              extensions.PerMessageDeflateExtensionProcessor)

    def test_minimal_offer(self):
        processor = extensions.PerMessageDeflateExtensionProcessor(
            common.ExtensionParameter('permessage-deflate'))

        response = processor.get_extension_response()
        self.assertEqual('permessage-deflate', response.name())
        self.assertEqual(0, len(response.get_parameters()))

        self.assertEqual(zlib.MAX_WBITS,
                         processor._rfc1979_deflater._window_bits)
        self.assertFalse(processor._rfc1979_deflater._no_context_takeover)

    def test_offer_with_max_window_bits(self):
        parameter = common.ExtensionParameter('permessage-deflate')
        parameter.add_parameter('server_max_window_bits', '10')
        processor = extensions.PerMessageDeflateExtensionProcessor(parameter)

        response = processor.get_extension_response()
        self.assertEqual('permessage-deflate', response.name())
        self.assertEqual([('server_max_window_bits', '10')],
                         response.get_parameters())

        self.assertEqual(10, processor._rfc1979_deflater._window_bits)

    def test_offer_with_out_of_range_max_window_bits(self):
        parameter = common.ExtensionParameter('permessage-deflate')
        parameter.add_parameter('server_max_window_bits', '0')
        processor = extensions.PerMessageDeflateExtensionProcessor(parameter)

        self.assertIsNone(processor.get_extension_response())

    def test_offer_with_max_window_bits_without_value(self):
        parameter = common.ExtensionParameter('permessage-deflate')
        parameter.add_parameter('server_max_window_bits', None)
        processor = extensions.PerMessageDeflateExtensionProcessor(parameter)

        self.assertIsNone(processor.get_extension_response())

    def test_offer_with_no_context_takeover(self):
        parameter = common.ExtensionParameter('permessage-deflate')
        parameter.add_parameter('server_no_context_takeover', None)
        processor = extensions.PerMessageDeflateExtensionProcessor(parameter)

        response = processor.get_extension_response()
        self.assertEqual('permessage-deflate', response.name())
        self.assertEqual([('server_no_context_takeover', None)],
                         response.get_parameters())

        self.assertTrue(processor._rfc1979_deflater._no_context_takeover)

    def test_offer_with_no_context_takeover_with_value(self):
        parameter = common.ExtensionParameter('permessage-deflate')
        parameter.add_parameter('server_no_context_takeover', 'foobar')
        processor = extensions.PerMessageDeflateExtensionProcessor(parameter)

        self.assertIsNone(processor.get_extension_response())

    def test_offer_with_unknown_parameter(self):
        parameter = common.ExtensionParameter('permessage-deflate')
        parameter.add_parameter('foo', 'bar')
        processor = extensions.PerMessageDeflateExtensionProcessor(parameter)

        self.assertIsNone(processor.get_extension_response())


class PerMessageDeflateExtensionProcessorBuildingTest(unittest.TestCase):
    """A unittest for checking that PerMessageDeflateExtensionProcessor builds
    a response based on specified options correctly.
    """
    def test_response_with_max_window_bits(self):
        parameter = common.ExtensionParameter('permessage-deflate')
        parameter.add_parameter('client_max_window_bits', None)
        processor = extensions.PerMessageDeflateExtensionProcessor(parameter)
        processor.set_client_max_window_bits(10)

        response = processor.get_extension_response()
        self.assertEqual('permessage-deflate', response.name())
        self.assertEqual([('client_max_window_bits', '10')],
                         response.get_parameters())

    def test_response_with_max_window_bits_without_client_permission(self):
        processor = extensions.PerMessageDeflateExtensionProcessor(
            common.ExtensionParameter('permessage-deflate'))
        processor.set_client_max_window_bits(10)

        response = processor.get_extension_response()
        self.assertIsNone(response)

    def test_response_with_true_for_no_context_takeover(self):
        processor = extensions.PerMessageDeflateExtensionProcessor(
            common.ExtensionParameter('permessage-deflate'))

        processor.set_client_no_context_takeover(True)

        response = processor.get_extension_response()
        self.assertEqual('permessage-deflate', response.name())
        self.assertEqual([('client_no_context_takeover', None)],
                         response.get_parameters())

    def test_response_with_false_for_no_context_takeover(self):
        processor = extensions.PerMessageDeflateExtensionProcessor(
            common.ExtensionParameter('permessage-deflate'))

        processor.set_client_no_context_takeover(False)

        response = processor.get_extension_response()
        self.assertEqual('permessage-deflate', response.name())
        self.assertEqual(0, len(response.get_parameters()))


if __name__ == '__main__':
    unittest.main()

# vi:sts=4 sw=4 et
