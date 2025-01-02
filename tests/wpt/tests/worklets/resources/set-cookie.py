def main(request, response):
    name = request.GET.first(b"name")
    value = request.GET.first(b"value")
    source_origin = request.headers.get(b"origin")
    if source_origin is None:
        # Same origin GET won't include origin header
        source_origin = "%s://%s" % (request.url_parts.scheme,
                                     request.url_parts.netloc)
        if request.url_parts.port:
            source_origin += ":%s" % request.url_parts.port

    response_headers = [(b"Set-Cookie", name + b"=" + value),
                        (b"Access-Control-Allow-Origin", source_origin),
                        (b"Access-Control-Allow-Credentials", b"true")]
    return (200, response_headers, u"")
