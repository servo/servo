#!/usr/bin/python

import sys, urllib, time
from mod_pywebsocket import common, msgutil, util

def web_socket_do_extra_handshake(request):
    request.connection.write('HTTP/1.1 101 Switching Protocols:\x0D\x0AConnection: Upgrade\x0D\x0AUpgrade: WebSocket\x0D\x0ASec-WebSocket-Origin: '+request.ws_origin+'\x0D\x0ASec-WebSocket-Accept: thisisawrongacceptkey\x0D\x0A\x0D\x0A')
    return

def web_socket_transfer_data(request):
    while True:
        request.ws_stream.send_message('test', binary=False)
        return
