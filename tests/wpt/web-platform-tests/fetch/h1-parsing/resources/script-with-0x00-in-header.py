def main(request, response):
    response.headers.set("Content-Type", "text/javascript")
    response.headers.set("Custom", "\0")
    return "var thisIsJavaScript = 0"
