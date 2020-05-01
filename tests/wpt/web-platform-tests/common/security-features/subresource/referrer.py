def main(request, response):
    referrer = request.headers.get("referer", "")
    response_headers = [("Content-Type", "text/javascript")];
    return (200, response_headers, "window.referrer = '" + referrer + "'")
