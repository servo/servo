def main(request, response):
    response.writer.write_status(200)
    response.writer.write_header(b"Content-Type", b"text/plain")
    response.writer.write_header(b"Connection", b"close")
    response.writer.end_headers()
    response.writer.write(str(request.raw_headers))
    response.close_connection = True
