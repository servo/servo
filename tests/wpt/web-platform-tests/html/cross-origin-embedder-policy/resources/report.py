import json


def main(request, response):
    if request.method == 'OPTIONS':
        # CORS preflight
        response.headers.set('Access-Control-Allow-Origin', '*')
        response.headers.set('Access-Control-Allow-Methods', 'POST')
        response.headers.set('Access-Control-Allow-Headers', 'content-type')
        return ''

    url_dir = '/'.join(request.url_parts.path.split('/')[:-1]) + '/'
    key = request.GET.first('key')
    reports = request.server.stash.take(key, url_dir) or []
    for report in json.loads(request.body):
        reports.append(report)
    request.server.stash.put(key, reports, url_dir)
    response.headers.set('Access-Control-Allow-Origin', '*')
    return 'done'
