import json

def main(request, response):
  key = request.GET.first(b'id')

  # No CORS support for cross-origin reporting endpoints
  if request.method == u'POST':
    reports = request.server.stash.take(key) or []
    for report in json.loads(request.body):
      reports.append(report)
    request.server.stash.put(key, reports)
    return b'done'
  if request.method == u'GET':
    return json.dumps(request.server.stash.take(key) or [])

  response.status = 400
  return b'invalid method'
