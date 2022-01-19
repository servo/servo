#!/usr/bin/python

from mod_pywebsocket import msgutil

def web_socket_do_extra_handshake(request):
    line = request.headers_in.get('sec-websocket-protocol')
    request.ws_protocol = line.split(',', 1)[0]

#pass

def web_socket_transfer_data(request):
    while True:
        msgutil.send_message(request, request.ws_protocol)
        return
