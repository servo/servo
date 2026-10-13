# Test runs following request chain:
#
# filter-no-referrer.html -> filter.css -> filter.py
#
# filter.py returns a filter resource to turn a rectangle green in
# filter-no-referrer.html if the referer header passed to filter.py does not
# contains "no-referrer". filter-no-referrer.html sets referrerpolicy to
# no-referrer, so that should definitely not leak to the svg resource request
# here.
#
def main(request, response):
    response.add_required_headers = False
    response.writer.write_status(200)
    response.writer.write_header(b"access-control-allow-origin", b"*")
    response.writer.write_header(b"content-type", b"image/svg+xml")
    response.writer.write_header(b"cache-control",
                                 b"no-cache; must-revalidate")

    if request.headers.get(b'referer',
                           b'').decode("utf-8").find('no-referrer') == -1:
        body = (
            b'<svg xmlns="http://www.w3.org/2000/svg">'
            b'  <defs>'
            b'    <filter id="make-solid-green" color-interpolation-filters="sRGB">'
            b'      <feColorMatrix type="matrix" values="'
            b'        0 0 0 0 0'
            b'        0 0 0 0 0.502'
            b'        0 0 0 0 0'
            b'        0 0 0 1 0" />'
            b'    </filter>'
            b'  </defs>'
            b'</svg>')
    else:
        body = b""

    response.writer.write_header(b"content-length", len(body))
    response.writer.end_headers()
    response.writer.write(body)
