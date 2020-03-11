import json


def main(request, response):
    if request.method == 'OPTIONS':
        # CORS preflight
        response.headers.set('Access-Control-Allow-Origin', '*')
        response.headers.set('Access-Control-Allow-Methods', 'POST')
        response.headers.set('Access-Control-Allow-Headers', 'content-type')
        return ''

    uuidMap = {
        'endpoint': '01234567-0123-0123-0123-0123456789AB',
        'report-only-endpoint': '01234567-0123-0123-0123-0123456789CD'
    }

    response.headers.set('Access-Control-Allow-Origin', '*')
    endpoint = request.GET.first('endpoint')
    if endpoint not in uuidMap:
        response.status = 400
        return 'invalid endpoint'

    path = '/'.join(request.url_parts.path.split('/')[:-1]) + '/'
    key = uuidMap[endpoint]

    if request.method == 'POST':
        reports = request.server.stash.take(key, path) or []
        for report in json.loads(request.body):
            reports.append(report)
        request.server.stash.put(key, reports, path)
        return 'done'

    if request.method == 'GET':
        return json.dumps(request.server.stash.take(key, path) or [])

    response.status = 400
    return 'invalid method'
