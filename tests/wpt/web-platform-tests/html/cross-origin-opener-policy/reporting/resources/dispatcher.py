# A server used to store and retrieve arbitrary data.
# This is used by: ./dispatcher.js
import json

def main(request, response):
    response.headers.set('Access-Control-Allow-Origin', '*')
    response.headers.set('Access-Control-Allow-Methods', 'OPTIONS, GET, POST')
    response.headers.set('Access-Control-Allow-Headers', 'Content-Type')
    response.headers.set('Cache-Control', 'no-cache, no-store, must-revalidate');
    if request.method == 'OPTIONS': # CORS preflight
        return ''

    uuid = request.GET['uuid']

    if request.method == 'POST':
        return request.server.stash.put(uuid, request.body)
    else:
        body = request.server.stash.take(uuid)
        if body is None:
            return 'not ready'
        else:
            return body
