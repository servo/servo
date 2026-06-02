def main(request, response):
    """
    Simple handler that sets the status based on whether sec-ch-device-memory was received.
    """
    response.headers.append(b"Access-Control-Allow-Origin", b"*")
    response.headers.append(b"Access-Control-Allow-Headers", b"*")
    response.headers.append(b"Access-Control-Expose-Headers", b"*")
    if b"sec-ch-device-memory" in request.headers:
        response.status = 200
    else:
        response.status = 400