#!/usr/bin/python

import time

# The amount of internal buffering a WebSocket connection has is not
# standardised, and varies depending upon the OS. Setting this number too small
# will result in false negatives, as the entire message gets buffered. Setting
# this number too large will result in false positives, when it takes more than
# 2 seconds to transmit the message anyway. This number was arrived at by
# trial-and-error.
MESSAGE_SIZE = 1024 * 1024

# With Windows 10 and Python 3, the OS will buffer an entire message in memory
# and return from send() immediately, even if it is very large. To work around
# this problem, send multiple messages.
MESSAGE_COUNT = 16


def web_socket_do_extra_handshake(request):
    # Turn off permessage-deflate, otherwise it shrinks our big message to a
    # tiny message.
    request.ws_extension_processors = []


def web_socket_transfer_data(request):
    # Send empty message to fill the ReadableStream queue
    request.ws_stream.send_message(b'', binary=True)

    # TODO(ricea@chromium.org): Use time.perf_counter() when migration to python
    # 3 is complete. time.time() can go backwards.
    start_time = time.time()

    # The large messages that will be blocked by backpressure.
    for i in range(MESSAGE_COUNT):
        request.ws_stream.send_message(b' ' * MESSAGE_SIZE, binary=True)

    # Report the time taken to send the large message.
    request.ws_stream.send_message(str(time.time() - start_time),
                                   binary=False)
