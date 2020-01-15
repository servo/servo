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
