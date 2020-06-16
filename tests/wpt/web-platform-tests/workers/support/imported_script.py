def main(request, response):
    return [(b'Content-Type', request.GET[b'mime'])], u""
