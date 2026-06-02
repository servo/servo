#!/usr/bin/python

from pywebsocket3 import msgutil

def web_socket_do_extra_handshake(request):
    pass

def web_socket_transfer_data(request):
    referrer = request.headers_in.get("referer")
    if referrer is None:
        referrer = "MISSING AS PER FETCH"
    msgutil.send_message(request, referrer)
