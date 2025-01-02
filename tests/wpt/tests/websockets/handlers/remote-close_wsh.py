# Copyright (c) 2024 The Chromium Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

"""
Perform a server-initiated close according to the parameters passed in the
query string. Supported parameters:

 * code=INT: The close code to send in the close frame. If omitted the Close
   frame will have an empty body.

 * reason=TEXT: The reason to be sent in the close frame. Only sent if `code` is
   set.

 * abrupt=1: Close the connection without sending a Close frame.

Example: /remote-close?code=1000&reason=Done

"""

import urllib

from pywebsocket3.handshake import AbortedByUserException


def web_socket_do_extra_handshake(request):
    pass


def web_socket_transfer_data(request):
    parts = urllib.parse.urlsplit(request.uri)
    parameters = urllib.parse.parse_qs(parts.query)
    if 'abrupt' in parameters:
        # Send a ping frame to make sure this isn't misinterpreted as a
        # handshake failure.
        request.ws_stream.send_ping('ping')
        # Rudely close the connection.
        raise AbortedByUserException('Abort the connection')
    code = None
    reason = None
    if 'code' in parameters:
        code = int(parameters['code'][0])
        reason = parameters.get('reason', [''])[0]
    request.ws_stream.close_connection(code, reason)
