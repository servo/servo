def main(request, response):
    headers = [("Content-Type", "text/javascript"), ("Cache-control", "public, max-age=100")]
    body = "throw('fox');"
    return 200, headers, body
