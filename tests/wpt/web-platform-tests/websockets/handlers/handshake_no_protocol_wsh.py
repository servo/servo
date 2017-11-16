#!/usr/bin/python

def web_socket_do_extra_handshake(request):
    # Trick pywebsocket into believing no subprotocol was requested.
    request.ws_requested_protocols = None

def web_socket_transfer_data(request):
    pass
