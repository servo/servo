#!/usr/bin/python

import sys, urllib, time
from mod_pywebsocket import common, msgutil, util


def web_socket_do_extra_handshake(request):
    msg = (b'HTTP/1.1 101 Switching Protocols:\x0D\x0A'
           b'Connection: Upgrade\x0D\x0A'
           b'Upgrade: WebSocket\x0D\x0A'
           b'Sec-WebSocket-Origin: %s\x0D\x0A'
           b'Sec-WebSocket-Accept: thisisawrongacceptkey\x0D\x0A\x0D\x0A') % request.ws_origin.encode('UTF-8')
    request.connection.write(msg)
    return


def web_socket_transfer_data(request):
    while True:
        request.ws_stream.send_message('test', binary=False)
        return
