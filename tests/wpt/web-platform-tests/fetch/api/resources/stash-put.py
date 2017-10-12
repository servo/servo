def main(request, response):
    url_dir = '/'.join(request.url_parts.path.split('/')[:-1]) + '/'
    key = request.GET.first("key")
    value = request.GET.first("value")
    request.server.stash.put(key, value, url_dir)
    response.headers.set('Access-Control-Allow-Origin', '*')
    return "done"
