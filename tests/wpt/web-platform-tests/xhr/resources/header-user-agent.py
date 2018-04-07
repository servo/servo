def main(request, response):
    response.headers.set("Access-Control-Allow-Origin", "*")
    response.headers.set("Access-Control-Max-Age", 0)
    response.headers.set('Access-Control-Allow-Headers', "x-test")

    if request.method == "OPTIONS":
        if not request.headers.get("User-Agent"):
            response.content = "FAIL: User-Agent header missing in preflight request."
            response.status = 400
    else:
        if request.headers.get("User-Agent"):
            response.content = "PASS"
        else:
            response.content = "FAIL: User-Agent header missing in request"
            response.status = 400
