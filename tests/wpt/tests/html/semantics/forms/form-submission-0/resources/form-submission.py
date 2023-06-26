def main(request, response):
    if request.headers.get(b'Content-Type') == b'application/x-www-form-urlencoded':
        result = request.body == b'foo=bara'
    elif request.headers.get(b'Content-Type') == b'text/plain':
        result = request.body == b'qux=baz\r\n'
    else:
        result = request.POST.first(b'foo') == b'bar'

    result = result and request.url_parts.query == u'query=1'

    return ([(b"Content-Type", b"text/plain")],
            b"OK" if result else b"FAIL")
