import time

def main(request, response):
    key = request.GET.first("key")

    if request.method == "POST":
        # Received result data from target page
        request.server.stash.put(key, request.body, '/scroll-to-text-fragment/')
        return "ok"
    else:
        # Request for result data from test page
        value = request.server.stash.take(key, '/scroll-to-text-fragment/')
        return value
