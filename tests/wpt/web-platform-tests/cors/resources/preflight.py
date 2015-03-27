def main(request, response):
    headers = [("Content-Type", "text/plain")]

    if "check" in request.GET:
        token = request.GET.first("token")
        value = request.server.stash.take(token)
        if value == None:
            body = "0"
        else:
            if request.GET.first("check", None) == "keep":
                request.server.stash.put(token, value)
            body = "1"

        return headers, body

    if request.method == "OPTIONS":
        if not "Access-Control-Request-Method" in request.headers:
            response.set_error(400, "No Access-Control-Request-Method header")
            return "ERROR: No access-control-request-method in preflight!"

        headers.append(("Access-Control-Allow-Methods",
                        request.headers['Access-Control-Request-Method']))

        if "max_age" in request.GET:
            headers.append(("Access-Control-Max-Age", request.GET['max_age']))

        if "token" in request.GET:
            request.server.stash.put(request.GET.first("token"), 1)

    headers.append(("Access-Control-Allow-Origin", "*"))
    headers.append(("Access-Control-Allow-Headers", "x-print"))

    body = request.headers.get("x-print", "NO")

    return headers, body
