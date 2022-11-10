def main(request, response):
    headers = [(b"Content-Type", b"text/javascript")]
    body = u'dprHeader = "%s";' % request.headers.get(b'sec-ch-dpr', '')
    return 200, headers, body
