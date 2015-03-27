#!/usr/bin/python

from mod_pywebsocket import msgutil, util

def web_socket_do_extra_handshake(request):
    request.ws_protocol = request.headers_in.get('Sec-WebSocket-Protocol')
#pass

def web_socket_transfer_data(request):
    while True:
        msgutil.send_message(request, request.ws_protocol)
        return
