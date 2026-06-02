#!/usr/bin/env python


def main(request, response):
    headers = []
    if request.headers.get(b'if-none-match', None):
        status = 304, u"Not Modified"
        return status, headers, u""
    else:
        headers.append((b"Content-Type", b"text/javascript"))
        headers.append((b"Cross-Origin-Embedder-Policy", b"require-corp"))
        headers.append((b"Cache-Control", b"private, max-age=0, must-revalidate"))
        headers.append((b"ETag", b"abcdef"))
        status = 200, u"OK"
        return status, headers, u"self.onmessage = (e) => { self.postMessage('LOADED'); };"
