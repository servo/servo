#!/usr/bin/python

from mod_pywebsocket import msgutil

def web_socket_do_extra_handshake(request):
    pass  # Always accept.

def web_socket_transfer_data(request):
    line = msgutil.receive_message(request)
    if line == "":
        msgutil.send_message(request, 'pass')
    else:
        msgutil.send_message(request, 'fail')
