def main(request, response):
    if "referrerPolicy" in request.GET:
        response.headers.set("Referrer-Policy",
                             request.GET.first("referrerPolicy"))
    response.status = 200
    response.headers.set("Content-Type", "text/html")
    response.content = "<meta charset=utf-8>\n<body><script>parent.postMessage('action','*')</script></body>"
