def main(request, response):
    # Disable non-standard XSS protection
    response.headers.set(b"X-XSS-Protection", b"0")
    response.headers.set(b"Content-Type", b"text/html")

    # Set caching headers with immutable directive
    # Cache-Control: immutable indicates the resource will never change,
    # and should be cached even when Pragma: no-cache is present.
    response.headers.set(b"Cache-Control", b"max-age=2592000, immutable")
    response.headers.set(b"Pragma", b"no-cache")

    # Include a timestamp to verify caching behavior
    import time
    response.content = f"Timestamp: {time.time()}".encode('utf-8')
