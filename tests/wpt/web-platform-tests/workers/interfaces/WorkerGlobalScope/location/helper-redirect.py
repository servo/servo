def main(request, response):
    response.status = 302
    response.headers.append("Location", "post-location-members.js?a")
