from urllib.parse import unquote

from wptserve.utils import isomorphic_encode

def main(request, response):
    response.headers.set(b'Set-Cookie', isomorphic_encode(unquote(request.url_parts.query)))
    return [(b"Content-Type", b"text/plain")], b""
