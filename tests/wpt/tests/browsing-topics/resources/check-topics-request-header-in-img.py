def main(request, response):
    """
    This file is intended to be requested twice to verify that the correct headers
    are included for images.
    1. Make an initial request for an img. The `sec-browsing-topics` header will
       be stored for step 2. The request will be redirected to an image.
    2. Make a request with the query parameter set. The stashed header from the
       first step will be returned in the response content.

    Parameters:
    `token` should be a unique UUID request parameter for the duration of this
    request. It will get stored in the server stash and will be used later in
    a query request.
    `query` should be a request parameter indicating the request would like
    to know the last `sec-browsing-topics` header with that token.
    """

    token = request.GET.first(b"token", None)
    is_query = request.GET.first(b"query", None) is not None
    topics_header = request.headers.get(b"sec-browsing-topics", b"NO_TOPICS_HEADER")

    queried_topics_header = b"NO_PREVIOUS_REQUEST"
    with request.server.stash.lock:
        value = request.server.stash.take(token)
        if value is not None:
            queried_topics_header = value
        if not is_query:
            request.server.stash.put(token, topics_header)

    if is_query:
        return (200, [(b"Access-Control-Allow-Origin", b"*")], queried_topics_header)

    headers = [(b"Location", "pixel.png"),
            (b"Access-Control-Allow-Origin", b"*")]
    return 301, headers, b""
