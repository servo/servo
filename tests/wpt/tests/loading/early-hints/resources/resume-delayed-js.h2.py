def main(request, response):
    id = request.GET.first(b"id")
    url_dir = u'/'.join(request.url_parts.path.split(u'/')[:-1]) + u'/'
    request.server.stash.put(id, True, url_dir)
    headers = [
        ("Content-Type", "text/plain"),
        ("Access-Control-Allow-Origin", "*"),
    ]
    body = "OK"
    return (200, "OK"), headers, body
