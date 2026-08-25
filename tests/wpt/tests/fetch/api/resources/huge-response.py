# A Python script that generates a huge response. Implemented as a script to
# avoid needing to add a huge file to the repository.

TOTAL_SIZE = 8 * 1024 * 1024 * 1024  # 8 GB
CHUNK_SIZE = 1024 * 1024  # 1 MB

assert TOTAL_SIZE % CHUNK_SIZE == 0


def main(request, response):
    response.headers.set(b"Content-type", b"text/plain")
    response.headers.set(b"Content-Length", str(TOTAL_SIZE).encode())
    response.headers.set(b"Cache-Control", b"max-age=86400")
    response.write_status_headers()

    chunk = bytes(CHUNK_SIZE)
    total_sent = 0

    while total_sent < TOTAL_SIZE:
        if not response.writer.write(chunk):
            break
        total_sent += CHUNK_SIZE
