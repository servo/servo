#!/usr/bin/python

from mod_pywebsocket import msgutil

def web_socket_do_extra_handshake(request):
    pass

def web_socket_transfer_data(request):
    referrer = request.headers_in.get("Referer")
    if referrer is None:
        referrer = "MISSING AS PER FETCH"
    msgutil.send_message(request, referrer)
