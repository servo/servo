#!/usr/bin/python

from pywebsocket3 import msgutil


def web_socket_do_extra_handshake(request):
    pass # Always accept.


def web_socket_transfer_data(request):
    msgutil.send_message(request, request.ws_origin)
