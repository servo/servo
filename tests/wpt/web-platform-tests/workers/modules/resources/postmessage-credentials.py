def main(request, response):
    cookie = request.cookies.first("COOKIE_NAME", None)

    response_headers = [("Content-Type", "text/javascript"),
                        ("Access-Control-Allow-Credentials", "true")]

    origin = request.headers.get("Origin", None)
    if origin:
        response_headers.append(("Access-Control-Allow-Origin", origin))

    cookie_value = '';
    if cookie:
        cookie_value = cookie.value;
    return (200, response_headers,
            "if ('DedicatedWorkerGlobalScope' in self &&" +
            "    self instanceof DedicatedWorkerGlobalScope) {" +
            "  postMessage('"+cookie_value+"');" +
            "} else if (" +
            "    'SharedWorkerGlobalScope' in self &&" +
            "    self instanceof SharedWorkerGlobalScope) {" +
            "  onconnect = e => e.ports[0].postMessage('"+cookie_value+"');" +
            "}")
