from six.moves.urllib import parse


def main(request, response):
    response.headers.set('Set-Cookie', parse.unquote(request.url_parts.query))
    return [("Content-Type", "text/plain")], ""
