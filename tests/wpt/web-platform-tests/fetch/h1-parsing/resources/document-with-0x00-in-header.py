def main(request, response):
    response.headers.set("Content-Type", "text/html")
    response.headers.set("Custom", "\0")
    return "<!doctype html><b>This is a document.</b>"
