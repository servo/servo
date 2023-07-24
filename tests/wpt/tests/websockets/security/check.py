def main(request, response):
    return b"FAIL" if b'Sec-WebSocket-Key' in request.headers else b"PASS"
