def main(request, response):
    status = int(request.GET.first(b"status", b"302"))
    headers = [(b"Location", b"")]
    return status, headers, b""
