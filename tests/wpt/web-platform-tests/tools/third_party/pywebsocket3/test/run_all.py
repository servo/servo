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
"""Run all tests in the same directory.

This suite is expected to be run under pywebsocket's src directory, i.e. the
directory containing mod_pywebsocket, test, etc.

To change loggin level, please specify --log-level option.
    python test/run_test.py --log-level debug

To pass any option to unittest module, please specify options after '--'. For
example, run this for making the test runner verbose.
    python test/run_test.py --log-level debug -- -v
"""

from __future__ import absolute_import
import logging
import optparse
import os
import re
import sys
import unittest

_TEST_MODULE_PATTERN = re.compile(r'^(test_.+)\.py$')


def _list_test_modules(directory):
    module_names = []
    for filename in os.listdir(directory):
        match = _TEST_MODULE_PATTERN.search(filename)
        if match:
            module_names.append(match.group(1))
    return module_names


def _suite():
    loader = unittest.TestLoader()
    return loader.loadTestsFromNames(
        _list_test_modules(os.path.join(os.path.split(__file__)[0], '.')))


if __name__ == '__main__':
    parser = optparse.OptionParser()
    parser.add_option(
        '--log-level',
        '--log_level',
        type='choice',
        dest='log_level',
        default='warning',
        choices=['debug', 'info', 'warning', 'warn', 'error', 'critical'])
    options, args = parser.parse_args()
    logging.basicConfig(level=logging.getLevelName(options.log_level.upper()),
                        format='%(levelname)s %(asctime)s '
                        '%(filename)s:%(lineno)d] '
                        '%(message)s',
                        datefmt='%H:%M:%S')
    unittest.main(defaultTest='_suite', argv=[sys.argv[0]] + args)

# vi:sts=4 sw=4 et
