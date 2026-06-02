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

    if b"access_control_allow_credentials_header" in request.GET:
      response.headers.append(b"Access-Control-Allow-Credentials", request.GET[b"access_control_allow_credentials_header"])

    if b"access_control_allow_origin_header" in request.GET:
      response.headers.append(b"Access-Control-Allow-Origin", request.GET[b"access_control_allow_origin_header"])

    if b"shared_storage_cross_origin_worklet_allowed_header" in request.GET:
      response.headers.append(b"Shared-Storage-Cross-Origin-Worklet-Allowed", request.GET[b"shared_storage_cross_origin_worklet_allowed_header"])

    if action == b"store-cookie":
      cookie = request.headers.get(b"Cookie", b"NO_COOKIE_HEADER")
      request.server.stash.put(token, cookie)
      return b""
    else:
      assert action == b"get-cookie"
      return request.server.stash.take(token)
