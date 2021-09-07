def main(request, response):
    if request.method == "OPTIONS":
        response.headers.set(b"Access-Control-Allow-Origin", b"*")
        response.headers.set(b"Access-Control-Allow-Headers", b"*")
        response.status = 200
    elif request.method == "GET":
        response.headers.set(b"Access-Control-Allow-Origin", b"*")
        if request.headers.get(b"X-Test"):
            response.headers.set(b"Content-Type", b"text/plain")
            response.content = b"PASS"
        else:
            response.status = 400
