# Returns a valid response when request's |referrer| matches
# |expected_referrer|.
def main(request, response):
    # We want |referrer| to be the referrer header with no query params,
    # because |expected_referrer| will not contain any query params, and
    # thus cannot be compared with the actual referrer header if it were to
    # contain query params. This works fine if the actual referrer has no
    # query params too.
    referrer = request.headers.get("referer", "").split("?")[0]
    referrer_policy = request.GET.first("referrer_policy")
    expected_referrer = request.GET.first("expected_referrer", "")
    response_headers = [("Content-Type", "text/javascript"),
                        ("Access-Control-Allow-Origin", "*")]

    if referrer_policy == "no-referrer" or referrer_policy == "origin":
        if referrer == expected_referrer:
            return (200, response_headers, "")
        return (404, response_headers)

    if referrer_policy == "same-origin":
        if referrer == expected_referrer:
            return (200, response_headers, "")
        return (404, response_headers)
    return (404, response_headers)
