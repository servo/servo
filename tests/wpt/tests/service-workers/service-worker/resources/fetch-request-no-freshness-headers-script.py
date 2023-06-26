def main(request, response):
    headers = []
    # Sets an ETag header to check the cache revalidation behavior.
    headers.append((b"ETag", b"abc123"))
    headers.append((b"Content-Type", b"text/javascript"))
    return headers, b"/* empty script */"
