import time

def main(request, response):
    response.headers.set(b"Content-Type", b"application/octet-stream")
    response.headers.set(b"Cache-Control", b"no-store")
    response.write_status_headers()

    response.writer.write_content(b"A" * 4096)
    time.sleep(0.1)
    response.writer.write_content(b"B" * 4096)
    time.sleep(0.1)
    response.writer.write_content(b"C" * 4096)
    time.sleep(5)
