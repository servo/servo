from wptserve.handlers import json_handler


@json_handler
def main(request, response):
    key = request.GET.first("key")
    return request.server.stash.take(key, '/fetch/range/')
