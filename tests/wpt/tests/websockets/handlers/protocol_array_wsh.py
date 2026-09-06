#!/usr/bin/python3

from pywebsocket3 import msgutil

def web_socket_do_extra_handshake(request):
    line = request.headers_in.get('sec-websocket-protocol')
    if line:
        request.ws_protocol = line.split(',', 1)[0]

def web_socket_transfer_data(request):
    message = request.ws_protocol or ''
    msgutil.send_message(request, message)
