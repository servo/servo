def main(request, response):
    chunks = ["First chunk\r\n",
              "Second chunk\r\n",
              "Yet another (third) chunk\r\n",
              "Yet another (fourth) chunk\r\n",
              ]
    response.headers.set("Transfer-Encoding", "chunked")
    response.headers.set("Trailer", "X-Test-Me")
    response.headers.set("Content-Type", "text/plain")
    response.write_status_headers()

    for value in chunks:
        response.writer.write("%x\r\n" % len(value))
        response.writer.write(value)
        response.writer.write("\r\n")
    response.writer.write("0\r\n")
    response.writer.write("X-Test-Me: Trailer header value\r\n\r\n")
