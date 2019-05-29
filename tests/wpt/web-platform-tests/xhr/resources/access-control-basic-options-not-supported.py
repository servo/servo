def main(request, response):
    response.headers.set("Cache-Control", "no-store")

    # Allow simple requests, but deny preflight
    if request.method != "OPTIONS":
        if "origin" in request.headers:
            response.headers.set("Access-Control-Allow-Credentials", "true")
            response.headers.set("Access-Control-Allow-Origin", request.headers["origin"])
        else:
            response.status = 500
    else:
        response.status = 400
