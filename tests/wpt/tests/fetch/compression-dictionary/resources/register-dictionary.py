import json

def main(request, response):
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    match = b"/fetch/compression-dictionary/resources/*"
    content = b"This is a test dictionary.\n"
    if b"match" in request.GET:
        match = request.GET.first(b"match")
    if b"content" in request.GET:
        content = request.GET.first(b"content")

    token = request.GET.first(b"save_header", None)
    if token is not None:
        headers = {}
        for header in request.headers:
            key = header.decode('utf-8')
            value = request.headers.get(header).decode('utf-8')
            headers[key] = value
        with request.server.stash.lock:
            request.server.stash.put(token, json.dumps(headers))

    previous_token = request.GET.first(b"get_previous_header", None)
    if previous_token is not None:
        result = {}
        with request.server.stash.lock:
            store = request.server.stash.take(previous_token)
            if store is not None:
                headers = json.loads(store)
                result["headers"] = headers
            return json.dumps(result)

    options = b"match=\"" + match + b"\""
    if b"id" in request.GET:
        options += b", id=\"" + request.GET.first(b"id") + b"\""
    response.headers.set(b"Use-As-Dictionary", options)
    response.headers.set(b"Cache-Control", b"max-age=3600")
    return content
