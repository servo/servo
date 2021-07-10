# Copyright 2013, Google Inc.
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
"""Handler for benchmark.html."""
from __future__ import absolute_import
import six


def web_socket_do_extra_handshake(request):
    # Turn off compression.
    request.ws_extension_processors = []


def web_socket_transfer_data(request):
    data = b''

    while True:
        command = request.ws_stream.receive_message()
        if command is None:
            return

        if not isinstance(command, six.text_type):
            raise ValueError('Invalid command data:' + command)
        commands = command.split(' ')
        if len(commands) == 0:
            raise ValueError('Invalid command data: ' + command)

        if commands[0] == 'receive':
            if len(commands) != 2:
                raise ValueError(
                    'Illegal number of arguments for send command' + command)
            size = int(commands[1])

            # Reuse data if possible.
            if len(data) != size:
                data = b'a' * size
            request.ws_stream.send_message(data, binary=True)
        elif commands[0] == 'send':
            if len(commands) != 2:
                raise ValueError(
                    'Illegal number of arguments for receive command' +
                    command)
            verify_data = commands[1] == '1'

            data = request.ws_stream.receive_message()
            if data is None:
                raise ValueError('Payload not received')
            size = len(data)

            if verify_data:
                if data != b'a' * size:
                    raise ValueError('Payload verification failed')

            request.ws_stream.send_message(str(size))
        else:
            raise ValueError('Invalid command: ' + commands[0])


# vi:sts=4 sw=4 et
