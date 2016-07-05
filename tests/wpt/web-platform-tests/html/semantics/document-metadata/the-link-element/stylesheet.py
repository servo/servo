def main(request, response):
    try:
        count = int(request.server.stash.take(request.GET["id"]))
    except:
        count = 0
    if "count" in request.GET:
        return str(count)
    request.server.stash.put(request.GET["id"], str(count + 1))
    return 'body { color: red }'
