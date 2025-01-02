#!/usr/bin/python

def web_socket_do_extra_handshake(request):
    request.ws_protocol = 'foobar'

def web_socket_transfer_data(request):
    pass