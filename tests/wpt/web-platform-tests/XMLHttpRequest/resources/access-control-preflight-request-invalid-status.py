def main(request, response):
    try:
        code = int(request.GET.first("code", None))
    except:
        code = None

    if request.method == "OPTIONS":
        if code:
            response.status = code
        response.headers.set("Access-Control-Max-Age", 1)
        response.headers.set("Access-Control-Allow-Headers", "x-pass")
    else:
        response.status = 200

    response.headers.set("Cache-Control", "no-store")
    response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))
