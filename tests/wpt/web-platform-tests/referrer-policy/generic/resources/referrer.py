def main(request, response):
    response_headers = [(b"Access-Control-Allow-Origin", b"*")]
    return (200, response_headers, request.headers.get("referer", ""))
