def main(request, response):
    # Return JSON with an invalid UTF-8 byte (0xFF) as the value
    # {"key":"X"} where X is 0xFF (invalid in UTF-8)
    response.headers.set(b"Content-Type", b"application/json")
    return b'{"key":"\xff"}'
