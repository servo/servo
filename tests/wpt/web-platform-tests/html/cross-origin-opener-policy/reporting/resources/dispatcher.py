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
    stash = request.server.stash;
    with stash.lock:
        queue = stash.take(uuid)
        if queue is None:
            queue = []

        if request.method == u'POST':
            queue.append(request.body)
            ret = b'done'
        else:
            if len(queue) == 0:
                ret = b'not ready'
            else:
                ret = queue.pop(0)
        stash.put(uuid, queue)
    return ret;
