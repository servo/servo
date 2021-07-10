def main(request, response):
    return [(b"Content-Type", b"text/html; charset=%s" % (request.GET[b'encoding']))], u""
