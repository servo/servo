def main(request, response):
    return 404, [(b"Content-Type", b"text/html")], b'Some content for the masses.' * 100
