import urllib

def main(request, response):
    response.headers.set('Set-Cookie', urllib.unquote(request.url_parts.query))
    return [("Content-Type", "text/plain")], ""
