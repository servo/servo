def main(request, response):
    response_ctype = ''

    if "response_charset_label" in request.GET:
      response_ctype = ";charset=" + request.GET.first("response_charset_label")

    headers = [("Content-type", "text/plain" + response_ctype),
               ("X-Request-Method", request.method),
               ("X-Request-Query", request.url_parts.query if request.url_parts.query else "NO"),
               ("X-Request-Content-Length", request.headers.get("Content-Length", "NO")),
               ("X-Request-Content-Type", request.headers.get("Content-Type", "NO"))]

    if "content" in request.GET:
        content = request.GET.first("content")
    else:
        content = request.body

    return headers, content
