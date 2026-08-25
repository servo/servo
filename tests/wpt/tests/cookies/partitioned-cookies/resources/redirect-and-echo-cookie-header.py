def main(request, response):
    # Performs a redirect to the location provided. Appends
    # the cookie headers to the url as query parameters.
    response.status = 302
    location = request.GET.first(b"location")
    cookies = b"?cookies=" + request.headers.get(b"cookie", b"")

    response.headers.set(b"Location", location + cookies)
