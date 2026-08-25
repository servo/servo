def main(request, response):
    response.headers.set(b"Content-Type", b"text/plain")
    response.headers.set(b"Refresh", b"0;./refreshed.txt?\x80\xFF") # Test byte to Unicode conversion
    response.content = u"Not refreshed.\n"
