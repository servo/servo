#!/usr/bin/python

"""A WebSocket handler that enforces basic HTTP authentication. Username is
'foo' and password is 'bar'."""


from mod_pywebsocket.handshake import AbortedByUserException


def web_socket_do_extra_handshake(request):
    authorization = request.headers_in.get('Authorization')
    if authorization is None or authorization != 'Basic Zm9vOmJhcg==':
        request.connection.write(
            'HTTP/1.1 401 Unauthorized\x0d\x0a'
            'Content-Length: 0\x0d\x0a'
            'WWW-Authenticate: Basic realm="camelot"\x0d\x0a'
            '\x0d\x0a')
        raise AbortedByUserException('Abort the connection')


def web_socket_transfer_data(request):
    pass
