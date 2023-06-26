import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    key = request.GET.first(b"stash")
    origin = request.headers.get(b"origin")
    if origin is None:
        origin = b"no Origin header"

    origin_list = request.server.stash.take(key)

    if b"dump" in request.GET:
        response.headers.set(b"Content-Type", b"application/json")
        response.content = json.dumps(origin_list)
        return

    if origin_list is None:
        origin_list = [isomorphic_decode(origin)]
    else:
        origin_list.append(isomorphic_decode(origin))

    request.server.stash.put(key, origin_list)

    if b"location" in request.GET:
        location = request.GET.first(b"location")
        if b"dummyJS" in request.GET:
            location += b"&dummyJS"
        response.status = 308
        response.headers.set(b"Location", location)
        return

    response.headers.set(b"Content-Type", b"text/html")
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    if b"dummyJS" in request.GET:
        response.content = b"console.log('dummy JS')"
    else:
        response.content = b"<meta charset=utf-8>\n<body><script>parent.postMessage('loaded','*')</script></body>"
