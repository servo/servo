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

from __future__ import absolute_import

from pywebsocket3 import common


def web_socket_do_extra_handshake(request):
    pass


def web_socket_transfer_data(request):
    while True:
        line = request.ws_stream.receive_message()
        if line is None:
            return
        code, reason = line.split(' ', 1)
        if code is None or reason is None:
            return
        request.ws_stream.close_connection(int(code), reason)
        # close_connection() initiates closing handshake. It validates code
        # and reason. If you want to send a broken close frame for a test,
        # following code will be useful.
        # > data = struct.pack('!H', int(code)) + reason.encode('UTF-8')
        # > request.connection.write(stream.create_close_frame(data))
        # > # Suppress to re-respond client responding close frame.
        # > raise Exception("customized server initiated closing handshake")


def web_socket_passive_closing_handshake(request):
    # Simply echo a close status code
    code, reason = request.ws_close_code, request.ws_close_reason

    # pywebsocket sets pseudo code for receiving an empty body close frame.
    if code == common.STATUS_NO_STATUS_RECEIVED:
        code = None
        reason = ''
    return code, reason


# vi:sts=4 sw=4 et
