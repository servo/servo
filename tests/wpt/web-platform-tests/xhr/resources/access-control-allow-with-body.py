def main(request, response):
    headers = {
        b"Cache-Control": b"no-store",
        b"Access-Control-Allow-Headers": b"X-Requested-With",
        b"Access-Control-Max-Age": 0,
        b"Access-Control-Allow-Origin": b"*",
        b"Access-Control-Allow-Methods": b"*",
        b"Vary": b"Accept-Encoding",
        b"Content-Type": b"text/plain"
    }

    for (name, value) in headers.items():
        response.headers.set(name, value)

    response.content = b"PASS"
