#!/usr/bin/python
from six.moves import urllib


def web_socket_do_extra_handshake(request):
    url_parts = urllib.parse.urlsplit(request.uri)
    request.extra_headers.append(('Set-Cookie', 'ws_test_'+(url_parts.query or '')+'=test; Secure; Path=/'))

def web_socket_transfer_data(request):
    # Expect close() from user agent.
    request.ws_stream.receive_message()
