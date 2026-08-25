def main(request, response):
    response.headers.set(b"Content-Type", b"text/html")
    response.headers.set(b"Refresh", request.GET.first(b"input"))
    response.content = u"<!doctype html>refresh.py\n"
