def main(request, response):
    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
    token = request.GET[b"token"]
    request.server.stash.put(token, b"")
    response.content = b"PASS"
