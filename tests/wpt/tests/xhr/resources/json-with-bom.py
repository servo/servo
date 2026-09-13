def main(request, response):
    # Return JSON with a UTF-8 BOM (0xEF 0xBB 0xBF) prefix
    response.headers.set(b"Content-Type", b"application/json")
    # BOM + {"key":"value"}
    return b'\xef\xbb\xbf{"key":"value"}'
