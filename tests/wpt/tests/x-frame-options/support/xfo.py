def main(request, response):
    headers = [(b"Content-Type", b"text/html"), (b"X-Frame-Options", request.GET.first(b"value"))]

    if b"value2" in request.GET:
        headers.append((b"X-Frame-Options", request.GET.first(b"value2")))

    if b"csp_value" in request.GET:
        headers.append((b"Content-Security-Policy", request.GET.first(b"csp_value")))

    body = u"""<!DOCTYPE html>
        <html>
        <head>
          <title>XFO.</title>
          <script>window.parent.postMessage('Loaded', '*');</script>
        </head>
        <body>
          Loaded
        </body>
        </html>
    """
    return (headers, body)
