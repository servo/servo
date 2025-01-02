import time
def main(request, response):
    response.headers.set(b"Content-Type", b"application/javascript")
    response.headers.set(b"Transfer-encoding", b"chunked")
    response.write_status_headers()

    time.sleep(1)

    response.writer.write(b"XX\r\n\r\n")
