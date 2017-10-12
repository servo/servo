def main(request, response):
    response.headers.set('Cache-Control', 'no-store')
    response.headers.set('Access-Control-Allow-Origin',
                         request.headers.get('origin'))

    headers = 'x-custom-s,x-custom-test,x-custom-u,x-custom-ua,x-custom-v'
    if request.method == 'OPTIONS':
        response.headers.set('Access-Control-Max-Age', '0')
        response.headers.set('Access-Control-Allow-Headers', headers)
        # Access-Control-Request-Headers should be sorted.
        if headers != request.headers.get('Access-Control-Request-Headers'):
            response.status = 400
    else:
        if request.headers.get('x-custom-s'):
            response.content = 'PASS'
        else:
            response.status = 400
            response.content = 'FAIL'
