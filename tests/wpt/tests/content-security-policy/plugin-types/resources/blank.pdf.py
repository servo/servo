def main(request, response):
    return [(b"Content-Type", b"application/pdf")], b"%PDF-1.0\n%%EOF\n"
