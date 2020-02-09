def main(request, response):
    if request.method == 'OPTIONS':
        # CORS preflight
        response.headers.set('Access-Control-Allow-Origin', '*')
        response.headers.set('Access-Control-Allow-Methods', '*')
        response.headers.set('Access-Control-Allow-Headers', '*')
        return 'done'

    url_dir = '/'.join(request.url_parts.path.split('/')[:-1]) + '/'
    key = request.GET.first("key")
    value = request.GET.first("value")
    request.server.stash.put(key, value, url_dir)
    response.headers.set('Access-Control-Allow-Origin', '*')
    return "done"
