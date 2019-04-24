def main(request, response):
    # Without X-XSS-Protection to disable non-standard XSS protection the functionality this
    # resource offers is useless
    response.headers.set("X-XSS-Protection", "0")
    response.headers.set("Content-Type", "text/html")
    response.content = request.GET.first("content")
