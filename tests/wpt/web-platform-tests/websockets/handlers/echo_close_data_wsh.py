#!/usr/bin/python
import six
from mod_pywebsocket import msgutil

_GOODBYE_MESSAGE = u'Goodbye'

def web_socket_do_extra_handshake(request):
    # This example handler accepts any request. See origin_check_wsh.py for how
    # to reject access from untrusted scripts based on origin value.

    pass  # Always accept.


def web_socket_transfer_data(request):
    while True:
        line = request.ws_stream.receive_message()
        if line is None:
            return
        if isinstance(line, six.text_type):
            if line == _GOODBYE_MESSAGE:
                return
                request.ws_stream.send_message(line, binary=False)
