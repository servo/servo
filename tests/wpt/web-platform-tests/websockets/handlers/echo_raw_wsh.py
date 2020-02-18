#!/usr/bin/python

from mod_pywebsocket import msgutil


def web_socket_do_extra_handshake(request):
    pass  # Always accept.

def web_socket_transfer_data(request):
    while True:
        line = msgutil.receive_message(request)
        if line == b'exit':
            return

        if line is not None:
            request.connection.write(line)
