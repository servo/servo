def main(request, response):
    if b"referrerPolicy" in request.GET:
        response.headers.set(b"Referrer-Policy",
                             request.GET.first(b"referrerPolicy"))
    response.status = 200
    response.headers.set(b"Content-Type", b"text/html")
    response.content = b"<meta charset=utf-8>\n<body><script>parent.postMessage('action','*')</script></body>"
