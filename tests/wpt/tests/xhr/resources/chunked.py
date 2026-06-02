def main(request, response):
    chunks = [b"First chunk\r\n",
              b"Second chunk\r\n",
              b"Yet another (third) chunk\r\n",
              b"Yet another (fourth) chunk\r\n",
             ]
    response.headers.set(b"Transfer-Encoding", b"chunked")
    response.headers.set(b"Trailer", b"X-Test-Me")
    response.headers.set(b"Content-Type", b"text/plain")
    response.write_status_headers()

    for value in chunks:
        response.writer.write(b"%x\r\n" % len(value))
        response.writer.write(value)
        response.writer.write(b"\r\n")
    response.writer.write(b"0\r\n")
    response.writer.write(b"X-Test-Me: Trailer header value\r\n\r\n")
