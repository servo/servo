"""
This generates a partial response for a 100-byte text file.
"""
import re

from wptserve.utils import isomorphic_decode

def main(request, response):
    total_length = int(request.GET.first(b'length', b'100'))
    partial_code = int(request.GET.first(b'partial', b'206'))
    content_type = request.GET.first(b'type', b'text/plain')
    range_header = request.headers.get(b'Range', b'')

    # Send a 200 if there is no range request
    if not range_header:
        to_send = ''.zfill(total_length)
        response.headers.set(b"Content-Type", content_type)
        response.headers.set(b"Cache-Control", b"no-cache")
        response.headers.set(b"Content-Length", total_length)
        response.content = to_send
        return

    # Simple range parsing, requires specifically "bytes=xxx-xxxx"
    range_header_match = re.search(r'^bytes=(\d*)-(\d*)$', isomorphic_decode(range_header))
    start, end = range_header_match.groups()
    start = int(start)
    end = int(end) if end else total_length
    length = end - start

    # Error the request if the range goes beyond the length
    if length <= 0 or end > total_length:
        response.set_error(416, u"Range Not Satisfiable")
        # set_error sets the MIME type to application/json, which - for a
        # no-cors media request - will be blocked by ORB. We'll just force
        # the expected MIME type here, whichfixes the test, but doesn't make
        # sense in general.
        response.headers = [(b"Content-Type", content_type)]
        response.write()
        return

    # Generate a partial response of the requested length
    to_send = ''.zfill(length)
    response.headers.set(b"Content-Type", content_type)
    response.headers.set(b"Accept-Ranges", b"bytes")
    response.headers.set(b"Cache-Control", b"no-cache")
    response.status = partial_code

    content_range = b"bytes %d-%d/%d" % (start, end, total_length)

    response.headers.set(b"Content-Range", content_range)
    response.headers.set(b"Content-Length", length)

    response.content = to_send
