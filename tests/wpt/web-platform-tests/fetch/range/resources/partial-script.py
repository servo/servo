"""
This generates a partial response containing valid JavaScript.
"""


def main(request, response):
    require_range = request.GET.first('require-range', '')
    pretend_offset = int(request.GET.first('pretend-offset', '0'))
    range_header = request.headers.get('Range', '')

    if require_range and not range_header:
        response.set_error(412, "Range header required")
        response.write()
        return

    response.headers.set("Content-Type", "text/plain")
    response.headers.set("Accept-Ranges", "bytes")
    response.headers.set("Cache-Control", "no-cache")
    response.status = 206

    to_send = 'self.scriptExecuted = true;'
    length = len(to_send)

    content_range = "bytes {}-{}/{}".format(
        pretend_offset, pretend_offset + length - 1, pretend_offset + length)

    response.headers.set("Content-Range", content_range)
    response.headers.set("Content-Length", length)

    response.content = to_send
