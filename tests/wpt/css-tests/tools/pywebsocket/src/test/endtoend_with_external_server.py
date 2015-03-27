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


"""Test for end-to-end with external server.

This test is not run by run_all.py because it requires some preparations.
If you would like to run this test correctly, launch Apache with mod_python
and mod_pywebsocket manually. In addition, you should pass allow_draft75 option
and example path as handler_scan option and Apache's DocumentRoot.
"""


import optparse
import sys
import test.test_endtoend
import unittest


_DEFAULT_WEB_SOCKET_PORT = 80


class EndToEndTestWithExternalServer(test.test_endtoend.EndToEndTest):
    pass

if __name__ == '__main__':
    parser = optparse.OptionParser()
    parser.add_option('-p', '--port', dest='port', type='int',
                      default=_DEFAULT_WEB_SOCKET_PORT,
                      help='external test server port.')
    (options, args) = parser.parse_args()

    test.test_endtoend._use_external_server = True
    test.test_endtoend._external_server_port = options.port

    unittest.main(argv=[sys.argv[0]])


# vi:sts=4 sw=4 et
