# Returns a worker script that posts the request's referrer header.
def main(request, response):
    referrer = request.headers.get(b"referer", b"")

    response_headers = [(b"Content-Type", b"text/javascript"),
                        (b"Access-Control-Allow-Origin", b"*")]

    return (200, response_headers,
            b"if ('DedicatedWorkerGlobalScope' in self &&" +
            b"    self instanceof DedicatedWorkerGlobalScope) {" +
            b"  postMessage('"+referrer+b"');" +
            b"} else if (" +
            b"    'SharedWorkerGlobalScope' in self &&" +
            b"    self instanceof SharedWorkerGlobalScope) {" +
            b"  onconnect = e => e.ports[0].postMessage('"+referrer+b"');" +
            b"}")
