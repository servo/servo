#!/usr/bin/python

def web_socket_do_extra_handshake(request):
    request.connection.write(b"FOO BAR BAZ\r\n\r\n")


def web_socket_transfer_data(request):
    pass
