def main(request, response):
    response.headers.set("Content-Type", "text/plain")
    response.headers.set("Refresh", "0;./refreshed.txt?\x80\xFF") # Test byte to Unicode conversion
    response.content = "Not refreshed.\n"
