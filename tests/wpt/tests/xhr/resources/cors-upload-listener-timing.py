import json


def main(request, response):
    response.headers.set(b"Cache-Control", b"no-store")
    key = request.GET.first(b"id")
    stash = request.server.stash

    if b"read" in request.GET:
        response.headers.set(b"Access-Control-Allow-Origin", b"*")
        response.headers.set(b"Content-Type", b"application/json")
        with stash.lock:
            return json.dumps(stash.take(key) or [])

    with stash.lock:
        methods = stash.take(key) or []
        methods.append(request.method)
        stash.put(key, methods)

    if request.GET.first(b"cors") == b"allow":
        response.headers.set(b"Access-Control-Allow-Origin", b"*")
        response.headers.set(b"Access-Control-Allow-Methods", b"POST")
        response.headers.set(b"Access-Control-Max-Age", b"0")

    response.headers.set(b"Content-Type", b"text/plain")
    if request.method == "OPTIONS":
        return b""
    return request.body
