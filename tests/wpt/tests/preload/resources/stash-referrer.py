from wptserve.utils import isomorphic_decode

def main(request, response):
    response.headers.set(b'Access-Control-Allow-Origin', b'*')

    # We assume this is a string representing a UUID
    key = request.GET.first(b'key')
    operation = request.GET.first(b'operation')

    if operation == b'put':
        referer = request.headers.get(b'referer') or 'NO-REFERER'
        request.server.stash.put(key, referer)
        return "Added value to stash"
    elif operation == b'take':
        value = request.server.stash.take(key)
        return value or ''
    else:
        assert False
