def main(request, response):
    if b'mime' in request.GET:
        return [(b'Content-Type', request.GET[b'mime'])], b""
    return [], b""
