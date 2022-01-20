def main(request, response):
    """
    Simple handler that sets a response header based on which client hint
    request headers were received.
    """

    response.headers.append(b"Access-Control-Allow-Origin", b"*")
    response.headers.append(b"Access-Control-Allow-Headers", b"*")
    response.headers.append(b"Access-Control-Expose-Headers", b"*")

    if b"sec-ch-device-memory" in request.headers:
            response.headers.set(b"device-memory-received", request.headers.get(b"sec-ch-device-memory"))
    if b"device-memory" in request.headers:
            response.headers.set(b"device-memory-deprecated-received", request.headers.get(b"device-memory"))
