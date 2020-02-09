#!/usr/bin/python

import time


def web_socket_do_extra_handshake(request):
    # Turn off permessage-deflate, otherwise it shrinks our 8MB buffer to 8KB.
    request.ws_extension_processors = []


def web_socket_transfer_data(request):
    # Wait two seconds to cause backpressure.
    time.sleep(2);
    request.ws_stream.receive_message()
