#!/usr/bin/python

from mod_pywebsocket import msgutil

def web_socket_do_extra_handshake(request):
    pass  # Always accept.

def web_socket_transfer_data(request):
    msgutil.send_message(request, request.unparsed_uri.split('?', 1)[1] or '')
