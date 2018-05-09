import helpers

def main(request, response):
    """Respond to `/cookie/set?{cookie}` by echoing `{cookie}` as a `Set-Cookie` header."""
    headers = helpers.setNoCacheAndCORSHeaders(request, response)
    headers.append(("Set-Cookie", request.url_parts.query))
    return headers, '{"success": true}'
