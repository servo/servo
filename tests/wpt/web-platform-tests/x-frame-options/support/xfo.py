def main(request, response):
    headers = [("Content-Type", "text/html"), ("X-Frame-Options", request.GET.first("value"))]

    if "value2" in request.GET:
      headers.append(("X-Frame-Options", request.GET.first("value2")))

    body = """<!DOCTYPE html>
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


