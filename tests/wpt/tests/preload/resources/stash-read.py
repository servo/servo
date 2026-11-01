from wptserve.utils import isomorphic_encode

def main(request, response):
    key = request.GET.first(b'key')
    value = request.server.stash.take(key, path="preload/resources/")
    return isomorphic_encode(value) if value is not None else b''