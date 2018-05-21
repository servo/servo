def main(request, response):
    response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))
    token = request.GET["token"]
    request.server.stash.put(token, "")
    response.content = "PASS"
