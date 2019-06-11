def main(request, response):
    """
    Simple handler that sets a response header based on which client hint
    request headers were received.
    """

    response.headers.append("Access-Control-Allow-Origin", "*")
    response.headers.append("Access-Control-Allow-Headers", "*")
    response.headers.append("Access-Control-Expose-Headers", "*")

    if "device-memory" in request.headers:
            response.headers.set("device-memory-received", request.headers.get("device-memory"))
    if "dpr" in request.headers:
            response.headers.set("dpr-received", request.headers.get("dpr"))
    if "viewport-width" in request.headers:
            response.headers.set("viewport-width-received", request.headers.get("viewport-width"))
    if "rtt" in request.headers:
            response.headers.set("rtt-received", request.headers.get("rtt"))
    if "downlink" in request.headers:
            response.headers.set("downlink-received", request.headers.get("downlink"))
    if "ect" in request.headers:
            response.headers.set("ect-received", request.headers.get("ect"))
    if "Sec-CH-Lang" in request.headers:
            response.headers.set("lang-received", request.headers.get("Sec-CH-Lang"))
