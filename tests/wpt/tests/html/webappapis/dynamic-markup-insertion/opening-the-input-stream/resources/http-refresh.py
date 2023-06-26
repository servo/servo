from wptserve.utils import isomorphic_encode

def main(request, response):
    time = isomorphic_encode(request.url_parts.query) if request.url_parts.query else b'0'
    return 200, [(b'Refresh', time), (b'Content-Type', b"text/html")], b''
