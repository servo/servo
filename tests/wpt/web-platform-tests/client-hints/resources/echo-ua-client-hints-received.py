def main(request, response):
    """
  Simple handler that sets a response header based on which client hint
  request headers were received.
  """

    response.headers.append(b"Access-Control-Allow-Origin", b"*")
    response.headers.append(b"Access-Control-Allow-Headers", b"*")
    response.headers.append(b"Access-Control-Expose-Headers", b"*")

    client_hint_headers = [
        b"sec-ch-ua",
        b"sec-ch-ua-arch",
        b"sec-ch-ua-platform",
        b"sec-ch-ua-platform-version",
        b"sec-ch-ua-model",
        b"sec-ch-ua-full-version",
    ]

    request_client_hints = {i: request.headers.get(i) for i in client_hint_headers}

    for header in client_hint_headers:
        if request_client_hints[header] is not None:
            response.headers.set(header + b"-recieved", request_client_hints[header])

    headers = []
    content = u""
    return 200, headers, content
