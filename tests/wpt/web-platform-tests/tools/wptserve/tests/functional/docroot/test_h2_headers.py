def handle_headers(frame, request, response):
    response.status = 203
    response.headers.update([('test', 'passed')])
