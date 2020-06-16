def main(request, response):
    response.status = 302
    response.headers.append(b"Location", b"post-location-members.js?a")
