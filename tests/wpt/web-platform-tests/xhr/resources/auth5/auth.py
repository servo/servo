from wptserve.utils import isomorphic_decode

def main(request, response):
    if request.auth.username == b'usr' and request.auth.password == b'secret':
        response.headers.set(b'Content-type', b'text/plain')
        content = b""
    else:
        response.status = 401
        response.headers.set(b'Status', b'401 Authorization required')
        response.headers.set(b'WWW-Authenticate', b'Basic realm="test"')
        content = b'User name/password wrong or not given: '

    content += b"%s\n%s" % (request.auth.username or b'',
                            request.auth.password or b'')
    return content
