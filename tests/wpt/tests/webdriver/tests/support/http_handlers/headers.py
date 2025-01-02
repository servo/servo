def main(request, response):
    """Simple handler that returns a response with custom headers.

    The request should define at least one "header" query parameter, with the
    format {key}:{value}. For instance ?header=foo:bar will create a response
    with a header with the key "foo" and the value "bar". Additional headers
    can be set by passing more "header" query parameters.
    """
    response.status = 200
    if b"header" in request.GET:
        try:
            headers = request.GET.get_list(b"header")
            for header in headers:
                header_parts = header.split(b":")
                response.headers.set(header_parts[0], header_parts[1])
        except ValueError:
            pass

    if b"Content-Type" not in response.headers:
        response.headers.set(b"Content-Type", "text/plain")

    response.content = "HTTP Response Headers"
