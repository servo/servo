import time


def main(request, response):
    headers = [
        (b"Access-Control-Allow-Origin", request.headers.get(b"Origin", b"*")),
        (b"Access-Control-Allow-Methods", b"GET"),
        (b"Access-Control-Allow-Headers", b"x-preflight-test"),
        (b"Access-Control-Max-Age", b"0"),
    ]

    if request.method != u"OPTIONS":
        headers.append((b"Content-Type", b"text/plain"))
        return 200, headers, b"actual request"

    response.status = 200
    for name, value in headers:
        response.headers.set(name, value)
    response.headers.set(b"Content-Type", b"text/plain")
    response.write_status_headers()

    while response.writer.write(b"." * 1024):
        time.sleep(0.01)
