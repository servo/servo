# A server used to store and retrieve arbitrary data.
# This is used by: ./dispatcher.js
def main(request, response):
    response.headers.set(b'Access-Control-Allow-Origin', b'*')
    response.headers.set(b'Access-Control-Allow-Methods', b'OPTIONS, GET, POST')
    response.headers.set(b'Access-Control-Allow-Headers', b'Content-Type')
    response.headers.set(b'Cache-Control', b'no-cache, no-store, must-revalidate')
    if request.method == u'OPTIONS': # CORS preflight
        return b''

    uuid = request.GET[b'uuid']
    stashed = request.server.stash.take(uuid)
    if stashed is None:
        stashed = []

    if request.method == u'POST':
        stashed.append(request.body)
        ret = b'done'
    else:
        if len(stashed) == 0:
            ret = b'not ready'
        else:
            ret = stashed.pop(0)
    request.server.stash.put(uuid, stashed)
    return ret;
