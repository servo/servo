def main(request, response):
    print request.GET['id']
    if request.method == 'POST':
        request.server.stash.put(request.GET["id"], request.body)
        return ''
    return request.server.stash.take(request.GET["id"])