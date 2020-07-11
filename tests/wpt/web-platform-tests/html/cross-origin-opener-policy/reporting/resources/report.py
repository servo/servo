import json, uuid

from six import PY3

from wptserve.utils import isomorphic_decode

def main(request, response):
    response.headers.set(b'Access-Control-Allow-Origin', b'*')
    response.headers.set(b'Access-Control-Allow-Methods', b'OPTIONS, GET, POST')
    response.headers.set(b'Access-Control-Allow-Headers', b'Content-Type')
    response.headers.set(b'Cache-Control', b'no-cache, no-store, must-revalidate');
    if request.method == u'OPTIONS': # CORS preflight
        return b''

    key = 0
    if b'endpoint' in request.GET:
        # Use Python version checking here due to the issue reported on uuid5 handling unicode
        # type of name argument at https://bugs.python.org/issue34145
        if PY3:
            key = uuid.uuid5(uuid.NAMESPACE_OID, isomorphic_decode(request.GET[b'endpoint'])).urn
        else:
            key = uuid.uuid5(uuid.NAMESPACE_OID, request.GET[b'endpoint']).urn

    if key == 0:
        response.status = 400
        return b'invalid endpoint'

    if request.method == u'POST':
        reports = request.server.stash.take(key) or []
        for report in json.loads(request.body):
            reports.append(report)
        request.server.stash.put(key, reports)
        return b"done"

    if request.method == u'GET':
        response.headers.set(b'Content-Type', b'application/json')
        return json.dumps(request.server.stash.take(key) or [])

    response.status = 400
    return b'invalid method'
