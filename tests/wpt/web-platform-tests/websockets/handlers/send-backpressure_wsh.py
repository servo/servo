#!/usr/bin/python

import time

# The amount of buffering a WebSocket connection has is not standardised, but
# it's reasonable to expect that it will not be as large as 8MB.
MESSAGE_SIZE = 8 * 1024 * 1024


def web_socket_do_extra_handshake(request):
    # Turn off permessage-deflate, otherwise it shrinks our 8MB buffer to 8KB.
    request.ws_extension_processors = []


def web_socket_transfer_data(request):
    # TODO(ricea@chromium.org): Use time.perf_counter() when migration to python
    # 3 is complete. time.time() can go backwards.
    start_time = time.time()
    request.ws_stream.send_message(b' ' * MESSAGE_SIZE, binary=True)
    request.ws_stream.send_message(str(time.time() - start_time), binary=False)
