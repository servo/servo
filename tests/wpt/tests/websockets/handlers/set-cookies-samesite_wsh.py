import urllib


def web_socket_do_extra_handshake(request):
    url_parts = urllib.parse.urlsplit(request.uri)
    max_age = ""
    if "clear" in url_parts.query:
        max_age = "; Max-Age=0"
    value = "1"
    if "value" in url_parts.query:
        value = urllib.parse.parse_qs(url_parts.query)["value"][0]
    cookies = [
        "samesite-unspecified={}; Path=/".format(value) + max_age,
        "samesite-lax={}; Path=/; SameSite=Lax".format(value) + max_age,
        "samesite-strict={}; Path=/; SameSite=Strict".format(value) + max_age,
        # SameSite=None cookies must be Secure.
        "samesite-none={}; Path=/; SameSite=None; Secure".format(value) + max_age
    ]
    for cookie in cookies:
        request.extra_headers.append(("Set-Cookie", cookie))


def web_socket_transfer_data(request):
    # Expect close() from user agent.
    request.ws_stream.receive_message()
