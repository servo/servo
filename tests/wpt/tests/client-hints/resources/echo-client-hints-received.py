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
    if b"sec-ch-dpr" in request.headers:
        response.headers.set(b"dpr-received", request.headers.get(b"sec-ch-dpr"))
    if b"dpr" in request.headers:
        response.headers.set(b"dpr-deprecated-received", request.headers.get(b"dpr"))
    if b"sec-ch-viewport-width" in request.headers:
        response.headers.set(b"viewport-width-received", request.headers.get(b"sec-ch-viewport-width"))
    if b"viewport-width" in request.headers:
        response.headers.set(b"viewport-width-deprecated-received", request.headers.get(b"viewport-width"))
    if b"sec-ch-viewport-height" in request.headers:
        response.headers.set(b"viewport-height-received", request.headers.get(b"sec-ch-viewport-height"))
    if b"rtt" in request.headers:
        response.headers.set(b"rtt-received", request.headers.get(b"rtt"))
    if b"downlink" in request.headers:
        response.headers.set(b"downlink-received", request.headers.get(b"downlink"))
    if b"ect" in request.headers:
        response.headers.set(b"ect-received", request.headers.get(b"ect"))
    if b"sec-ch-ua-mobile" in request.headers:
        response.headers.set(b"mobile-received", request.headers.get(b"sec-ch-ua-mobile"))
    if b"sec-ch-prefers-color-scheme" in request.headers:
        response.headers.set(b"prefers-color-scheme-received", request.headers.get(b"sec-ch-prefers-color-scheme"))
    if b"sec-ch-prefers-reduced-motion" in request.headers:
        response.headers.set(b"prefers-reduced-motion-received", request.headers.get(b"sec-ch-prefers-reduced-motion"))
    if b"sec-ch-prefers-reduced-transparency" in request.headers:
        response.headers.set(b"prefers-reduced-transparency-received", request.headers.get(b"sec-ch-prefers-reduced-transparency"))
