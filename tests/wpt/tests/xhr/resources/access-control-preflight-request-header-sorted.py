def main(request, response):
    response.headers.set(b'Cache-Control', b'no-store')
    response.headers.set(b'Access-Control-Allow-Origin',
                         request.headers.get(b'origin'))

    headers = b'x-custom-s,x-custom-test,x-custom-u,x-custom-ua,x-custom-v'
    if request.method == u'OPTIONS':
        response.headers.set(b'Access-Control-Max-Age', b'0')
        response.headers.set(b'Access-Control-Allow-Headers', headers)
        # Access-Control-Request-Headers should be sorted.
        if headers != request.headers.get(b'Access-Control-Request-Headers'):
            response.status = 400
    else:
        if request.headers.get(b'x-custom-s'):
            response.content = b'PASS'
        else:
            response.status = 400
            response.content = b'FAIL'
