def main(request, response):
    directive = request.GET.first(b"directive")
    response.headers.set(b"Clear-Site-Data", b"\"" + directive + b"\"")
    return b"OK"
