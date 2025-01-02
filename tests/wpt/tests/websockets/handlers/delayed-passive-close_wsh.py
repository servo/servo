#!/usr/bin/python
from pywebsocket3 import common
import time

def web_socket_do_extra_handshake(request):
    pass


def web_socket_transfer_data(request):
    # Wait for the close frame to arrive.
    request.ws_stream.receive_message()


def web_socket_passive_closing_handshake(request):
    # Echo close status code and reason
    code, reason = request.ws_close_code, request.ws_close_reason

    # No status received is a reserved pseudo code representing an empty code,
    # so echo back an empty code in this case.
    if code == common.STATUS_NO_STATUS_RECEIVED:
        code = None

    # The browser may error the connection if the closing handshake takes too
    # long, but hopefully no browser will have a timeout this short.
    time.sleep(1)

    return code, reason
