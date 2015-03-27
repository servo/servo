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


import unittest
import zlib

import set_sys_path  # Update sys.path to locate mod_pywebsocket module.

from mod_pywebsocket import common
from mod_pywebsocket import extensions


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
        self.assertRaises(
                ValueError, extensions._parse_window_bits, '10000000')


class CompressionMethodParameterParserTest(unittest.TestCase):
    """A unittest for _parse_compression_method which parses the compression
    method description used by perframe-compression and permessage-compression
    extension in their "method" extension parameter.
    """

    def test_parse_method_simple(self):
        method_list = extensions._parse_compression_method('foo')
        self.assertEqual(1, len(method_list))
        method = method_list[0]
        self.assertEqual('foo', method.name())
        self.assertEqual(0, len(method.get_parameters()))

    def test_parse_method_with_parameter(self):
        method_list = extensions._parse_compression_method('foo; x; y=10')
        self.assertEqual(1, len(method_list))
        method = method_list[0]
        self.assertEqual('foo', method.name())
        self.assertEqual(2, len(method.get_parameters()))
        self.assertTrue(method.has_parameter('x'))
        self.assertEqual(None, method.get_parameter_value('x'))
        self.assertTrue(method.has_parameter('y'))
        self.assertEqual('10', method.get_parameter_value('y'))

    def test_parse_method_with_quoted_parameter(self):
        method_list = extensions._parse_compression_method(
            'foo; x="Hello World"; y=10')
        self.assertEqual(1, len(method_list))
        method = method_list[0]
        self.assertEqual('foo', method.name())
        self.assertEqual(2, len(method.get_parameters()))
        self.assertTrue(method.has_parameter('x'))
        self.assertEqual('Hello World', method.get_parameter_value('x'))
        self.assertTrue(method.has_parameter('y'))
        self.assertEqual('10', method.get_parameter_value('y'))

    def test_parse_method_multiple(self):
        method_list = extensions._parse_compression_method('foo, bar')
        self.assertEqual(2, len(method_list))
        self.assertEqual('foo', method_list[0].name())
        self.assertEqual(0, len(method_list[0].get_parameters()))
        self.assertEqual('bar', method_list[1].name())
        self.assertEqual(0, len(method_list[1].get_parameters()))

    def test_parse_method_multiple_methods_with_quoted_parameter(self):
        method_list = extensions._parse_compression_method(
            'foo; x="Hello World", bar; y=10')
        self.assertEqual(2, len(method_list))
        self.assertEqual('foo', method_list[0].name())
        self.assertEqual(1, len(method_list[0].get_parameters()))
        self.assertTrue(method_list[0].has_parameter('x'))
        self.assertEqual('Hello World',
                         method_list[0].get_parameter_value('x'))
        self.assertEqual('bar', method_list[1].name())
        self.assertEqual(1, len(method_list[1].get_parameters()))
        self.assertTrue(method_list[1].has_parameter('y'))
        self.assertEqual('10', method_list[1].get_parameter_value('y'))

    def test_create_method_desc_simple(self):
        params = common.ExtensionParameter('foo')
        desc = extensions._create_accepted_method_desc('foo',
                                                       params.get_parameters())
        self.assertEqual('foo', desc)

    def test_create_method_desc_with_parameters(self):
        params = common.ExtensionParameter('foo')
        params.add_parameter('x', 'Hello, World')
        params.add_parameter('y', '10')
        desc = extensions._create_accepted_method_desc('foo',
                                                       params.get_parameters())
        self.assertEqual('foo; x="Hello, World"; y=10', desc)


class DeflateFrameExtensionProcessorParsingTest(unittest.TestCase):
    """A unittest for checking that DeflateFrameExtensionProcessor parses given
    extension parameter correctly.
    """

    def test_registry(self):
        processor = extensions.get_extension_processor(
                common.ExtensionParameter('deflate-frame'))
        self.assertIsInstance(processor,
                              extensions.DeflateFrameExtensionProcessor)

        processor = extensions.get_extension_processor(
                common.ExtensionParameter('x-webkit-deflate-frame'))
        self.assertIsInstance(processor,
                              extensions.DeflateFrameExtensionProcessor)

    def test_minimal_offer(self):
        processor = extensions.DeflateFrameExtensionProcessor(
            common.ExtensionParameter('perframe-deflate'))

        response = processor.get_extension_response()
        self.assertEqual('perframe-deflate', response.name())
        self.assertEqual(0, len(response.get_parameters()))

        self.assertEqual(zlib.MAX_WBITS,
                         processor._rfc1979_deflater._window_bits)
        self.assertFalse(processor._rfc1979_deflater._no_context_takeover)

    def test_offer_with_max_window_bits(self):
        parameter = common.ExtensionParameter('perframe-deflate')
        parameter.add_parameter('max_window_bits', '10')
        processor = extensions.DeflateFrameExtensionProcessor(parameter)

        response = processor.get_extension_response()
        self.assertEqual('perframe-deflate', response.name())
        self.assertEqual(0, len(response.get_parameters()))

        self.assertEqual(10, processor._rfc1979_deflater._window_bits)

    def test_offer_with_out_of_range_max_window_bits(self):
        parameter = common.ExtensionParameter('perframe-deflate')
        parameter.add_parameter('max_window_bits', '0')
        processor = extensions.DeflateFrameExtensionProcessor(parameter)

        self.assertIsNone(processor.get_extension_response())

    def test_offer_with_max_window_bits_without_value(self):
        parameter = common.ExtensionParameter('perframe-deflate')
        parameter.add_parameter('max_window_bits', None)
        processor = extensions.DeflateFrameExtensionProcessor(parameter)

        self.assertIsNone(processor.get_extension_response())

    def test_offer_with_no_context_takeover(self):
        parameter = common.ExtensionParameter('perframe-deflate')
        parameter.add_parameter('no_context_takeover', None)
        processor = extensions.DeflateFrameExtensionProcessor(parameter)

        response = processor.get_extension_response()
        self.assertEqual('perframe-deflate', response.name())
        self.assertEqual(0, len(response.get_parameters()))

        self.assertTrue(processor._rfc1979_deflater._no_context_takeover)

    def test_offer_with_no_context_takeover_with_value(self):
        parameter = common.ExtensionParameter('perframe-deflate')
        parameter.add_parameter('no_context_takeover', 'foobar')
        processor = extensions.DeflateFrameExtensionProcessor(parameter)

        self.assertIsNone(processor.get_extension_response())

    def test_offer_with_unknown_parameter(self):
        parameter = common.ExtensionParameter('perframe-deflate')
        parameter.add_parameter('foo', 'bar')
        processor = extensions.DeflateFrameExtensionProcessor(parameter)

        response = processor.get_extension_response()
        self.assertEqual('perframe-deflate', response.name())
        self.assertEqual(0, len(response.get_parameters()))


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


class PerMessageCompressExtensionProcessorTest(unittest.TestCase):
    def test_registry(self):
        processor = extensions.get_extension_processor(
                common.ExtensionParameter('permessage-compress'))
        self.assertIsInstance(processor,
                              extensions.PerMessageCompressExtensionProcessor)


if __name__ == '__main__':
    unittest.main()


# vi:sts=4 sw=4 et
