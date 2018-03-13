def main(request, response):
    response.headers.set("Content-Type", "text/event-stream")
    return ""
