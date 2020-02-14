# Returns a worker script that posts the request's referrer header.
def main(request, response):
    referrer = request.headers.get("referer", "")

    response_headers = [("Content-Type", "text/javascript"),
                        ("Access-Control-Allow-Origin", "*")]

    return (200, response_headers,
            "if ('DedicatedWorkerGlobalScope' in self &&" +
            "    self instanceof DedicatedWorkerGlobalScope) {" +
            "  postMessage('"+referrer+"');" +
            "} else if (" +
            "    'SharedWorkerGlobalScope' in self &&" +
            "    self instanceof SharedWorkerGlobalScope) {" +
            "  onconnect = e => e.ports[0].postMessage('"+referrer+"');" +
            "}")
