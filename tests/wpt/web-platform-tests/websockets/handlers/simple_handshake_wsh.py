#!/usr/bin/python

from mod_pywebsocket import common, stream
from mod_pywebsocket.handshake import AbortedByUserException, hybi


def web_socket_do_extra_handshake(request):
    # Send simple response header. This test implements the handshake
    # manually. It's not clear why.
    msg = (b'HTTP/1.1 101 Switching Protocols:\x0D\x0A'
           b'Connection: Upgrade\x0D\x0A'
           b'Upgrade: WebSocket\x0D\x0A'
           b'Set-Cookie: ws_test=test\x0D\x0A'
           b'Sec-WebSocket-Origin: %s\x0D\x0A'
           b'Sec-WebSocket-Accept: %s\x0D\x0A\x0D\x0A') % (request.ws_origin.encode(
               'UTF-8'), hybi.compute_accept_from_unicode(request.headers_in.get(common.SEC_WEBSOCKET_KEY_HEADER)))
    request.connection.write(msg)
    # Send a clean close frame.
    body = stream.create_closing_handshake_body(1000, '')
    request.connection.write(stream.create_close_frame(body))
    # Wait for the responding close frame from the user agent. It's not possible
    # to use the stream methods at this point because the stream hasn't been
    # established from pywebsocket's point of view. Instead just read the
    # appropriate number of bytes and assume they are correct.
    request.connection.read(8)
    # Close the socket without pywebsocket sending its own handshake response.
    raise AbortedByUserException('Abort the connection')


def web_socket_transfer_data(request):
    pass
