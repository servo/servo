import os
import re
import time

from wptserve.utils import isomorphic_decode

BYTE_RANGE_RE = re.compile(r"bytes=(\d+)-(\d+)?$")


def main(request, response):
    chunk_size = int(request.GET.first(b"chunksize", b"1024"))
    chunk_delay = float(request.GET.first(b"chunkdelay", b"16")) / 1E3

    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), u"linearized.pdf")
    with open(path, u"rb") as file:
        content = file.read()

    total_length = len(content)
    first_byte = 0
    last_byte = total_length - 1

    range_header = request.headers.get(b"Range", b"")
    if range_header:
        match = BYTE_RANGE_RE.match(isomorphic_decode(range_header))
        if not match:
            response.status = 416
            return b""
        first_byte = int(match.group(1))
        if match.group(2) is not None:
            last_byte = min(int(match.group(2)), last_byte)
        response.status = 206
        response.headers.set(b"Content-Range", b"bytes %d-%d/%d" % (first_byte, last_byte, total_length))
    else:
        response.status = 200

    content = content[first_byte:last_byte + 1]

    response.headers.set(b"Content-Type", b"application/pdf")
    response.headers.set(b"Accept-Ranges", b"bytes")
    response.headers.set(b"Content-Length", b"%d" % len(content))
    response.headers.set(b"Cache-Control", b"no-cache, no-store, must-revalidate")
    response.write_status_headers()

    for offset in range(0, len(content), chunk_size):
        response.writer.write_content(content[offset:offset + chunk_size])
        time.sleep(chunk_delay)
