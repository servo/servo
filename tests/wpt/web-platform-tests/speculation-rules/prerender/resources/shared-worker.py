def main(request, response):
    if b"check" in request.GET:
        with request.server.stash.lock:
            result = request.server.stash.take(request.GET[b"id"])
            response.headers.set(b"Content-Type", b"text/plain")
            return result
    else:
        with request.server.stash.lock:
            request.server.stash.put(request.GET[b"id"], "ok")
            response.headers.set(b"Content-Type", b"text/javascript")
        return u"onconnect = ({ports: [port]}) => port.postMessage(performance.timeOrigin);"