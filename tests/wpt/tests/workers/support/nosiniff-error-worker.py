def main(request, response):
    return [(b'Content-Type', b'text/html'),
            (b'X-Content-Type-Options', b'nosniff')], u""
