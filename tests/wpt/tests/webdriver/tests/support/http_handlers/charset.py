def main(request, response):
    """Simple handler that will return a text/plain response without any charset
    information by default.

    The request should define a "content" query parameter, which will be the
    response text content to use.

    Optionally you can provide a "charset" parameter which will be appended to
    the content-type header.
    """
    response.status = 200
    if b"charset" in request.GET:
        charset = request.GET.first(b"charset").decode("utf-8")
        response.headers.set(b"Content-Type", f"text/plain; charset={charset}")
    else:
        response.headers.set(b"Content-Type", "text/plain")


    response.content = request.GET.first(b"content")
