# Returns a valid response when a request has appropriate credentials.
def main(request, response):
    cookie = request.cookies.first(b"cookieName", None)
    expected_value = request.GET.first(b"value", None)
    source_origin = request.headers.get(b"origin", None)
    if source_origin is None:
        # Same origin GET won't include origin header
        source_origin = "%s://%s" % (request.url_parts.scheme,
                                     request.url_parts.netloc)
        if request.url_parts.port:
            source_origin += ":%s" % request.url_parts.port

    response_headers = [(b"Content-Type", b"text/javascript"),
                        (b"Access-Control-Allow-Origin", source_origin),
                        (b"Access-Control-Allow-Credentials", b"true")]

    if cookie == expected_value:
        return (200, response_headers, u"")

    return (404, response_headers, u"")
