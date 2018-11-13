def main(request, response):
  service_worker_header = request.headers.get('service-worker')
  if service_worker_header == 'script':
    body = '// Request has `Service-Worker: script` header'
    return 200, [('Content-Type', 'application/javascript')], body
  else:
    return 400, [('Content-Type', 'text/plain')], 'Bad Request'
