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

    if request.method == u'POST':
        return request.server.stash.put(uuid, request.body)
    else:
        body = request.server.stash.take(uuid)
        if body is None:
            return b'not ready'
        else:
            return body
