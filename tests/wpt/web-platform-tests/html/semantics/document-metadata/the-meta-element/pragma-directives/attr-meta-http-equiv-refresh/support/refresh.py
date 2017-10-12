def main(request, response):
    response.headers.set("Content-Type", "text/html")
    response.headers.set("Refresh", request.GET.first("input"))
    response.content = "<!doctype html>refresh.py\n"
