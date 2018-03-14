def main(request, response):
    """
    Simple handler that sets a response header based on if device-memory
    request header was received or not.
    """

    if "device-memory" in request.headers:
            response.headers.set("device-memory-received", "true")