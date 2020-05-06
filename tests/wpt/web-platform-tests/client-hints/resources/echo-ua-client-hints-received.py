def main(request, response):
    """
  Simple handler that sets a response header based on which client hint
  request headers were received.
  """

    response.headers.append("Access-Control-Allow-Origin", "*")
    response.headers.append("Access-Control-Allow-Headers", "*")
    response.headers.append("Access-Control-Expose-Headers", "*")

    client_hint_headers = [
        "sec-ch-ua",
        "sec-ch-ua-arch",
        "sec-ch-ua-platform",
        "sec-ch-ua-platform-version",
        "sec-ch-ua-model",
        "sec-ch-ua-full-version",
    ]

    request_client_hints = {i: request.headers.get(i) for i in client_hint_headers}

    for header in client_hint_headers:
        if request_client_hints[header] is not None:
            response.headers.set(header + "-recieved", request_client_hints[header])

    headers = []
    content = ""
    return 200, headers, content
