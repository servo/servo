from wptserve.handlers import json_handler


@json_handler
def main(request, response):
    dir = '/'.join(request.url_parts.path.split('/')[:-1]) + '/'
    key = request.GET.first("key")
    response.headers.set('Access-Control-Allow-Origin', '*')
    return request.server.stash.take(key, dir)
