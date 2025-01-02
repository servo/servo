def main(request, response):
    return int(request.GET[b"status"]), [], b""
