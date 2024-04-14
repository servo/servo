def main(request, response):
    response.status = (200, b"OK")
    response.headers.set(b"Ad-Auction-Allowed", b"true")
    response.headers.set(b"Content-Type", b"application/json")
    body = request.GET.first(b"body", None)
    return body