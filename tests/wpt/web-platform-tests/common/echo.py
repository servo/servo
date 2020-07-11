def main(request, response):
    # Without X-XSS-Protection to disable non-standard XSS protection the functionality this
    # resource offers is useless
    response.headers.set(b"X-XSS-Protection", b"0")
    response.headers.set(b"Content-Type", b"text/html")
    response.content = request.GET.first(b"content")
