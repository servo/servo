def main(request, response):
    """Simple handler that returns a response with Cache-Control max-age=3600.
    """

    status = int(request.GET.get(b"status", None))
    # For redirects, a "location" get parameter can indicate the redirected url
    if status == 301 and b"location" in request.GET:
        response.headers.set(b"Location", request.GET.first(b"location"))

    response.status = status


    if b"contenttype" in request.GET:
        content_type = request.GET.first(b"contenttype")
        response.headers.set(b"Content-Type", content_type)
    else:
        response.headers.set(b"Content-Type", "text/plain")

    response.headers.set(b"Expires", "Thu, 01 Dec 2100 20:00:00 GMT")
    response.headers.set(b"Cache-Control", "max-age=3600")

    if b"response" in request.GET:
        return request.GET.first(b"response")
    else:
        return "Cached HTTP Response"
