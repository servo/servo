import json, uuid

def main(request, response):
    response.headers.set('Access-Control-Allow-Origin', '*')
    response.headers.set('Access-Control-Allow-Methods', 'OPTIONS, GET, POST')
    response.headers.set('Access-Control-Allow-Headers', 'Content-Type')
    response.headers.set('Cache-Control', 'no-cache, no-store, must-revalidate');
    if request.method == 'OPTIONS': # CORS preflight
        return ''

    key = 0;
    if 'endpoint' in request.GET:
        key = uuid.uuid5(uuid.NAMESPACE_OID, request.GET['endpoint']).get_urn()

    if key == 0:
        response.status = 400
        return 'invalid endpoint'

    if request.method == 'POST':
        reports = request.server.stash.take(key) or []
        for report in json.loads(request.body):
            reports.append(report)
        request.server.stash.put(key, reports)
        return "done"

    if request.method == 'GET':
        response.headers.set('Content-Type', 'application/json')
        return json.dumps(request.server.stash.take(key) or [])

    response.status = 400
    return 'invalid method'
