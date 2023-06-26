def main(request, response):
    if request.method == "OPTIONS":
        response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
        response.headers.set(b"Access-Control-Allow-Headers", b"X-Test")
        response.status = 200
    elif request.method == "GET":
        response.headers.set(b"Access-Control-Allow-Origin", b"*")
        if request.headers.get(b"X-Test"):
            response.headers.set(b"Content-Type", b"text/plain")
            response.content = "PASS"
        else:
            response.status = 400
