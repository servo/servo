def main(request, response):
    headers = {
        "Cache-Control": "no-store",
        "Access-Control-Allow-Headers": "X-Requested-With",
        "Access-Control-Max-Age": 0,
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "*",
        "Vary": "Accept-Encoding",
        "Content-Type": "text/plain"
    }

    for (name, value) in headers.items():
        response.headers.set(name, value)

    response.content = "PASS"
