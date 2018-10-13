def main(request, response):
  service_worker_header = request.headers.get('service-worker')

  if 'header' in request.GET and service_worker_header != 'script':
    return 400, [('Content-Type', 'text/plain')], 'Bad Request'

  if 'no-header' in request.GET and service_worker_header == 'script':
    return 400, [('Content-Type', 'text/plain')], 'Bad Request'

  # no-cache itself to ensure the user agent finds a new version for each
  # update.
  headers = [('Cache-Control', 'no-cache, must-revalidate'),
             ('Pragma', 'no-cache'),
             ('Content-Type', 'application/javascript')]
  body = '/* This is a service worker script */\n'

  if 'import' in request.GET:
    body += "importScripts('%s');" % request.GET['import']

  return 200, headers, body
