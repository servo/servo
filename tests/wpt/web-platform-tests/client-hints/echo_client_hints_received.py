def main(request, response):
    """
    Simple handler that sets a response header based on if device-memory
    request header was received or not.
    """

    if "device-memory" in request.headers:
            response.headers.set("device-memory-received", "true")
    if "dpr" in request.headers:
            response.headers.set("dpr-received", "true")
    if "viewport-width" in request.headers:
            response.headers.set("viewport-width-received", "true")