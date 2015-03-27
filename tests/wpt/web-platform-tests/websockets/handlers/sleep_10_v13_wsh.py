#!/usr/bin/python

import sys, urllib, time
from mod_pywebsocket import common, msgutil, util

def web_socket_do_extra_handshake(request):
    request.connection.write('x')
    time.sleep(2)
    request.connection.write('x')
    time.sleep(2)
    request.connection.write('x')
    time.sleep(2)
    request.connection.write('x')
    time.sleep(2)
    request.connection.write('x')
    time.sleep(2)
    return

def web_socket_transfer_data(request):
    while True:
        line = msgutil.receive_message(request)
        if line == 'Goodbye':
            return
        request.ws_stream.send_message(line, binary=False)

