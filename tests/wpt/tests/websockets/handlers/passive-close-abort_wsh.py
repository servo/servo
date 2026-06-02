# Copyright (c) 2024 The Chromium Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Wait for a Close frame from the client and then close the connection without
sending a Close frame in return.
"""

from pywebsocket3.handshake import AbortedByUserException


def web_socket_do_extra_handshake(request):
    pass


def web_socket_transfer_data(request):
    while True:
        if request.ws_stream.receive_message() is None:
            return


def web_socket_passive_closing_handshake(request):
    raise AbortedByUserException('abrupt close')
