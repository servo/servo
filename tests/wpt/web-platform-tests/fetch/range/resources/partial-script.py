"""
This generates a partial response containing valid JavaScript.
"""

def main(request, response):
    require_range = request.GET.first(b'require-range', b'')
    pretend_offset = int(request.GET.first(b'pretend-offset', b'0'))
    range_header = request.headers.get(b'Range', b'')

    if require_range and not range_header:
        response.set_error(412, u"Range header required")
        response.write()
        return

    response.headers.set(b"Content-Type", b"text/plain")
    response.headers.set(b"Accept-Ranges", b"bytes")
    response.headers.set(b"Cache-Control", b"no-cache")
    response.status = 206

    to_send = b'self.scriptExecuted = true;'
    length = len(to_send)

    content_range = b"bytes %d-%d/%d" % (
        pretend_offset, pretend_offset + length - 1, pretend_offset + length)

    response.headers.set(b"Content-Range", content_range)
    response.headers.set(b"Content-Length", length)

    response.content = to_send
