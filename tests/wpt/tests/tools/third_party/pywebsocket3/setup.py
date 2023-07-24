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
"""Set up script for mod_pywebsocket.
"""

from __future__ import absolute_import
from __future__ import print_function
from setuptools import setup, Extension
import sys

_PACKAGE_NAME = 'mod_pywebsocket'

# Build and use a C++ extension for faster masking. SWIG is required.
_USE_FAST_MASKING = False

# This is used since python_requires field is not recognized with
# pip version 9.0.0 and earlier
if sys.hexversion < 0x020700f0:
    print('%s requires Python 2.7 or later.' % _PACKAGE_NAME, file=sys.stderr)
    sys.exit(1)

if _USE_FAST_MASKING:
    setup(ext_modules=[
        Extension('mod_pywebsocket/_fast_masking',
                  ['mod_pywebsocket/fast_masking.i'],
                  swig_opts=['-c++'])
    ])

setup(
    author='Yuzo Fujishima',
    author_email='yuzo@chromium.org',
    description='Standalone WebSocket Server for testing purposes.',
    long_description=('mod_pywebsocket is a standalone server for '
                      'the WebSocket Protocol (RFC 6455). '
                      'See mod_pywebsocket/__init__.py for more detail.'),
    license='See LICENSE',
    name=_PACKAGE_NAME,
    packages=[_PACKAGE_NAME, _PACKAGE_NAME + '.handshake'],
    python_requires='>=2.7',
    install_requires=['six'],
    url='https://github.com/GoogleChromeLabs/pywebsocket3',
    version='3.0.1',
)

# vi:sts=4 sw=4 et
