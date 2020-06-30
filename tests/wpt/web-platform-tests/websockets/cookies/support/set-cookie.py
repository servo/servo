from six.moves.urllib import parse

from wptserve.utils import isomorphic_encode

def main(request, response):
    response.headers.set(b'Set-Cookie', isomorphic_encode(parse.unquote(request.url_parts.query)))
    return [(b"Content-Type", b"text/plain")], b""
