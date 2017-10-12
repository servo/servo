def main(request, response):
    if request.method == "OPTIONS" and request.cookies.get("foo"):
        response.status = 400
    else:
        response.headers.set("Cache-Control", "no-store")
        response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))
        response.headers.set("Access-Control-Allow-Credentials", "true")
        response.headers.set("Access-Control-Allow-Headers", "X-Proprietary-Header")
        response.headers.set("Connection", "close")

        if request.cookies.get("foo"):
            response.content = request.cookies["foo"].value
