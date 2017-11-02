# Returns a valid response when request's |referrer| matches |referrer_policy|.
def main(request, response):
    referrer = request.headers.get("referer", None)
    referrer_policy = request.GET.first("referrer_policy")
    source_origin = request.GET.first("source_origin")
    is_cross_origin = request.GET.first("is_cross_origin", False)

    response_headers = [("Content-Type", "text/javascript"),
                        ("Access-Control-Allow-Origin", source_origin)];

    # When the referrer policy is "no-referrer", the referrer header shouldn't
    # be sent.
    if referrer_policy == "no-referrer" and not referrer:
        return (200, response_headers, "")

    # When the referrer policy is "origin", the referrer header should contain
    # only the origin. Note that |referrer| contains a trailing slash, while
    # |source_origin| doesn't.
    if referrer_policy == "origin" and referrer == source_origin + "/":
        return (200, response_headers, "")

    # When the referrer policy is "same-origin", the referrer header should be
    # sent only for a same-origin request.
    if referrer_policy == "same-origin":
        if is_cross_origin and not referrer:
            return (200, response_headers, "")
        if not is_cross_origin and referrer:
            return (200, response_headers, "")

    return (404)
