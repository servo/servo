def main(request, response):
    """
    Simple handler that sets a response header based on if which client hint
    request headers were received.
    """

    response.headers.append("Access-Control-Allow-Origin", "*")

    if "device-memory" in request.headers:
            response.headers.set("device-memory-received", "true")
    if "dpr" in request.headers:
            response.headers.set("dpr-received", "true")
    if "viewport-width" in request.headers:
            response.headers.set("viewport-width-received", "true")
    if "rtt" in request.headers:
            response.headers.set("rtt-received", "true")
    if "downlink" in request.headers:
            response.headers.set("downlink-received", "true")
    if "ect" in request.headers:
            response.headers.set("ect-received", "true")