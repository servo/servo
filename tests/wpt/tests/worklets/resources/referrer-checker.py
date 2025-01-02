# Returns a valid response when request's |referrer| matches
# |expected_referrer|.
def main(request, response):
    # We want |referrer| to be the referrer header with no query params,
    # because |expected_referrer| will not contain any query params, and
    # thus cannot be compared with the actual referrer header if it were to
    # contain query params. This works fine if the actual referrer has no
    # query params too.
    referrer = request.headers.get(b"referer", b"").split(b"?")[0]
    referrer_policy = request.GET.first(b"referrer_policy")
    expected_referrer = request.GET.first(b"expected_referrer", b"")
    response_headers = [(b"Content-Type", b"text/javascript"),
                        (b"Access-Control-Allow-Origin", b"*")]

    if referrer_policy == b"no-referrer" or referrer_policy == b"origin":
        if referrer == expected_referrer:
            return (200, response_headers, u"")
        return (404, response_headers)

    if referrer_policy == b"same-origin":
        if referrer == expected_referrer:
            return (200, response_headers, u"")
        return (404, response_headers)
    return (404, response_headers)
