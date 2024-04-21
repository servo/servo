#!/usr/bin/python

from pywebsocket3 import msgutil

def web_socket_do_extra_handshake(request):
    request.ws_cookie = request.headers_in.get('cookie')

def web_socket_transfer_data(request):
    if request.ws_cookie is not None:
        msgutil.send_message(request, request.ws_cookie)
    else:
        msgutil.send_message(request, '(none)')
