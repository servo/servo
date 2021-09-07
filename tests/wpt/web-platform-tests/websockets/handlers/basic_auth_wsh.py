#!/usr/bin/python

"""A WebSocket handler that enforces basic HTTP authentication. Username is
'foo' and password is 'bar'."""


from mod_pywebsocket.handshake import AbortedByUserException


def web_socket_do_extra_handshake(request):
    authorization = request.headers_in.get('authorization')
    if authorization is None or authorization != 'Basic Zm9vOmJhcg==':
        if request.protocol == "HTTP/2":
            request.status = 401
            request.headers_out["Content-Length"] = "0"
            request.headers_out['www-authenticate'] = 'Basic realm="camelot"'
        else:
            request.connection.write(b'HTTP/1.1 401 Unauthorized\x0d\x0a'
                                     b'Content-Length: 0\x0d\x0a'
                                     b'WWW-Authenticate: Basic realm="camelot"\x0d\x0a'
                                     b'\x0d\x0a')
        raise AbortedByUserException('Abort the connection')


def web_socket_transfer_data(request):
    pass
