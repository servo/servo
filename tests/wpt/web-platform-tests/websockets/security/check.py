def main(request, response):
    return "FAIL" if 'Sec-WebSocket-Key' in request.headers else "PASS"
