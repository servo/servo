#!/usr/bin/python
import six
from mod_pywebsocket import msgutil
from mod_pywebsocket import common

_GOODBYE_MESSAGE = u'Goodbye'

def web_socket_do_extra_handshake(request):
    # This example handler accepts any request. See origin_check_wsh.py for how
    # to reject access from untrusted scripts based on origin value.
    if request.ws_requested_protocols:
        if "echo" in request.ws_requested_protocols:
            request.ws_protocol = "echo"


def web_socket_transfer_data(request):
    while True:
        line = request.ws_stream.receive_message()
        if line is None:
            return
        if isinstance(line, six.text_type):
            request.ws_stream.send_message(line, binary=False)
            if line == _GOODBYE_MESSAGE:
                return
        else:
            request.ws_stream.send_message(line, binary=True)

def web_socket_passive_closing_handshake(request):
    # Echo close status code and reason
    code, reason = request.ws_close_code, request.ws_close_reason

    # No status received is a reserved pseudo code representing an empty code,
    # so echo back an empty code in this case.
    if code == common.STATUS_NO_STATUS_RECEIVED:
        code = None

    return code, reason
