def main(request, response):
    path = request.GET["path"] if "path" in request.GET else None
    if request.method == 'POST':
         request.server.stash.put(request.GET["id"], request.body, path)
         return ''

    return request.server.stash.take(request.GET["id"], path)
