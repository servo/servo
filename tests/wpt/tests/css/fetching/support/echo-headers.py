import sys
import json

def main(request, response):
    token = request.GET.first(b"token", None)
    location = request.GET.first(b"location", None)
    store = request.server.stash.take(token)
    headers = {}
    if (location == b"echo"):
        return store

    if (store == None):
        store = {}
    else:
        store = json.loads(store)

    for header in request.headers:
        headers[header.decode('utf-8')] = request.headers.get(header).decode('utf-8')

    store[location.decode('utf-8')] = headers

    request.server.stash.put(token, json.dumps(store))
    response.status = 302
    response.headers.set(b"Location", location)
