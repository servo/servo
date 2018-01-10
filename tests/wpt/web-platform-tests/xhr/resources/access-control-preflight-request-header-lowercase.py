def main(request, response):
    response.headers.set("Cache-Control", "no-store")
    response.headers.set("Access-Control-Allow-Origin", "*")
    response.headers.set("Access-Control-Max-Age", 0)

    if request.method == "OPTIONS":
        if "x-test" in [header.strip(" ") for header in
                        request.headers.get("Access-Control-Request-Headers").split(",")]:
            response.headers.set("Access-Control-Allow-Headers", "X-Test")
        else:
            response.status = 400
    elif request.method == "GET":
        if request.headers.get("X-Test"):
            response.content = "PASS"
        else:
            response.status = 400
