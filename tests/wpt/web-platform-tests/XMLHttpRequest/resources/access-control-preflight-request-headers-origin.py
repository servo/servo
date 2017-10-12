def main(request, response):
    response.headers.set("Cache-Control", "no-store")
    response.headers.set("Access-Control-Allow-Origin", "*")

    if request.method == "OPTIONS":
        if "origin" in request.headers.get("Access-Control-Request-Headers").lower():
            response.status = 400
            response.content = "Error: 'origin' included in Access-Control-Request-Headers"
        else:
            response.headers.set("Access-Control-Allow-Headers", "x-pass")
    else:
        response.content = request.headers.get("x-pass")
