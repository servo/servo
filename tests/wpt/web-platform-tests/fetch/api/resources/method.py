def main(request, response):
    headers = []
    if "cors" in request.GET:
        headers.append(("Access-Control-Allow-Origin", "*"))
        headers.append(("Access-Control-Allow-Credentials", "true"))
        headers.append(("Access-Control-Allow-Methods", "GET, POST, PUT, FOO"))
        headers.append(("Access-Control-Allow-Headers", "x-test, x-foo"))
        headers.append(("Access-Control-Expose-Headers", "x-request-method"))

    headers.append(("x-request-method", request.method))
    headers.append(("x-request-content-type", request.headers.get("Content-Type", "NO")))
    headers.append(("x-request-content-length", request.headers.get("Content-Length", "NO")))
    headers.append(("x-request-content-encoding", request.headers.get("Content-Encoding", "NO")))
    headers.append(("x-request-content-language", request.headers.get("Content-Language", "NO")))
    headers.append(("x-request-content-location", request.headers.get("Content-Location", "NO")))
    return headers, request.body
