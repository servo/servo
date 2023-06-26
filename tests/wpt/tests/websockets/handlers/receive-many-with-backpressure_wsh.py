# Sleep to build backpressure, receive messages, and send back their length.
# Used by send-many-64K-messages-with-backpressure.any.js.


import time


def web_socket_do_extra_handshake(request):
    # Compression will interfere with backpressure, so disable the
    # permessage-delate extension.
    request.ws_extension_processors = []


def web_socket_transfer_data(request):
    while True:
        # Don't read the message immediately, so backpressure can build.
        time.sleep(0.1)
        line = request.ws_stream.receive_message()
        if line is None:
            return
        # Send back the size of the message as acknowledgement that it was
        # received.
        request.ws_stream.send_message(str(len(line)), binary=False)
