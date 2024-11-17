def main(_request, response):
    response.headers.set(b"Content-Type", b"image/svg+xml")
    response.headers.set(b"Refresh", b"0;./refreshed.txt") # Test byte to Unicode conversion
    response.content = u'<svg version="1.1" xmlns="http://www.w3.org/2000/svg"><text y="14" text-anchor="left" >Not refreshed.</text></svg>\n'
