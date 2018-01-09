# Returns a valid response when request's |referrer| matches
# |expected_referrer|.
def main(request, response):
    referrer = request.headers.get("referer", "")
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
        # The expected referrer doesn't contain query params for simplification,
        # so we check the referrer by startswith() here.
        if (expected_referrer != "" and
            referrer.startswith(expected_referrer + "?")):
            return (200, response_headers, "")
        return (404, response_headers)

    return (404, response_headers)
