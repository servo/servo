#!/usr/bin/python

from pywebsocket3 import msgutil

def web_socket_do_extra_handshake(request):
    request.ws_protocol = request.headers_in.get('sec-websocket-protocol')
#pass

def web_socket_transfer_data(request):
    while True:
        msgutil.send_message(request, request.ws_protocol)
        return
