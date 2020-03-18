def main(request, response):
    origin = request.headers.get("origin")

    if origin is not None:
        response.headers.set("Access-Control-Allow-Origin", origin)
        response.headers.set("Access-Control-Allow-Methods", "GET")
        response.headers.set("Access-Control-Allow-Credentials", "true")

    if request.method == "OPTIONS":
        return ""

    headers = [("Content-Type", "text/javascript")]
    milk = request.cookies.first("milk", None)

    if milk is None:
        return headers, "var included = false;"
    elif milk.value == "yes":
        return headers, "var included = true;"

    return headers, "var included = false;"
