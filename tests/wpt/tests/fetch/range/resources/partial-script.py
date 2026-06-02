"""
This generates a partial response containing valid JavaScript or image data.
"""

def main(request, response):
    require_range = request.GET.first(b'require-range', b'')
    pretend_offset = int(request.GET.first(b'pretend-offset', b'0'))
    range_not_satisfiable = request.GET.first(b'range-not-satisfiable', b'')
    content_type = request.GET.first(b'type', b'text/plain')
    range_header = request.headers.get(b'Range', b'')

    if require_range and not range_header:
        response.set_error(412, u"Range header required")
        response.write()
        return

    # 1x1 red PNG image (67 bytes)
    png_data = b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\xf8\xcf\xc0\x00\x00\x00\x03\x00\x01\x00\x18\xdd\x8d\xb4\x00\x00\x00\x00IEND\xaeB`\x82'

    if content_type == b'image/png':
        to_send = png_data
    else:
        to_send = b'self.scriptExecuted = true;'

    length = len(to_send)

    response.headers.set(b"Content-Type", content_type)
    response.headers.set(b"Accept-Ranges", b"bytes")
    response.headers.set(b"Cache-Control", b"no-cache")
    response.headers.set(b"Content-Length", length)

    if range_not_satisfiable:
        response.status = 416
        response.headers.set(b"Content-Range", b"bytes */%d" % (pretend_offset + length))
    else:
        response.status = 206
        content_range = b"bytes %d-%d/%d" % (
            pretend_offset, pretend_offset + length - 1, pretend_offset + length)
        response.headers.set(b"Content-Range", content_range)

    response.content = to_send
