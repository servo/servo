import json

def main(request, response):
    key = request.GET.first("stash")
    origin = request.headers.get("origin")
    if origin is None:
        origin = "no Origin header"

    origin_list = request.server.stash.take(key)

    if "dump" in request.GET:
        response.headers.set("content-Type", "application/json")
        response.content = json.dumps(origin_list)
        return

    if origin_list is None:
        origin_list = [origin]
    else:
        origin_list.append(origin)

    request.server.stash.put(key, origin_list)

    if "location" in request.GET:
        response.status = 308
        response.headers.set("Location", request.GET.first("location"))
        return

    response.headers.set("content-Type", "text/plain")
    response.content = "Fix https://github.com/whatwg/fetch/issues/737..."
