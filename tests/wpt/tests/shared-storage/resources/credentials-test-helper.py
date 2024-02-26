def main(request, response):
    """
    A handler that does either one of the following based on the provided
    "action" parameter:
    1) action="store-cookie": Stores the provided token and the request cookie
    to the stash, and returns a regular module script content.
    2) action="get-cookie": Retrieves and returns the content stored in the
    stash at the provided token.
    """
    token = request.GET[b"token"]
    action = request.GET[b"action"]

    response.status = 200
    response.headers.append(b"Content-Type", b"text/javascript")

    if action == b"store-cookie":
      cookie = request.headers.get(b"Cookie", b"NO_COOKIE_HEADER")
      request.server.stash.put(token, cookie)
      return b""
    else:
      assert action == b"get-cookie"
      return request.server.stash.take(token)
