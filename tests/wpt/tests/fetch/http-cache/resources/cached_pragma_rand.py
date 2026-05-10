def main(request, response):
    # Disable non-standard XSS protection
    response.headers.set(b"X-XSS-Protection", b"0")
    response.headers.set(b"Content-Type", b"text/html")

    # Set caching headers
    # According to rfc9111 Pragma: no-cache is deprecated, so we expect
    # Cache-Control to take precedence when there's a mismatch.
    response.headers.set(b"Cache-Control", b"max-age=2592000, public")
    response.headers.set(b"Pragma", b"no-cache")

    # Include a unique token to verify caching behavior
    import uuid
    response.content = f"Token: {uuid.uuid4()}".encode('utf-8')
