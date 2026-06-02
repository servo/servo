def main(request, response):
    """Simple handler that returns a response with Cache-Control max-age=0 and
    must-revalidate.
    The request can include a return-304 header to trigger the handler to return
    a 304 instead of a 200.
    """
    response.headers.set(b"Content-Type", "text/plain")

    if b"true" == request.headers.get(b"return-304", None):
        # instruct the browser that the response was not modified and the cache
        # can be used.
        response.status = 304
        return ""
    else:
        response.headers.set(b"Cache-Control", b"max-age=0, must-revalidate")
        response.status = 200
        return "must-revalidate HTTP Response"
