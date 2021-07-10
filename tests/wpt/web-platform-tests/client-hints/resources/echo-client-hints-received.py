def main(request, response):
    """
    Simple handler that sets a response header based on which client hint
    request headers were received.
    """

    response.headers.append(b"Access-Control-Allow-Origin", b"*")
    response.headers.append(b"Access-Control-Allow-Headers", b"*")
    response.headers.append(b"Access-Control-Expose-Headers", b"*")

    if b"device-memory" in request.headers:
        response.headers.set(b"device-memory-received", request.headers.get(b"device-memory"))
    if b"dpr" in request.headers:
        response.headers.set(b"dpr-received", request.headers.get(b"dpr"))
    if b"viewport-width" in request.headers:
        response.headers.set(b"viewport-width-received", request.headers.get(b"viewport-width"))
    if b"rtt" in request.headers:
        response.headers.set(b"rtt-received", request.headers.get(b"rtt"))
    if b"downlink" in request.headers:
        response.headers.set(b"downlink-received", request.headers.get(b"downlink"))
    if b"ect" in request.headers:
        response.headers.set(b"ect-received", request.headers.get(b"ect"))
    if b"Sec-CH-Lang" in request.headers:
        response.headers.set(b"lang-received", request.headers.get(b"Sec-CH-Lang"))
    if b"sec-ch-ua-mobile" in request.headers:
        response.headers.set(b"mobile-received", request.headers.get(b"sec-ch-ua-mobile"))
