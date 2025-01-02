def main(request, response):
  # This script generates a worker script for static imports from module
  # service workers.
  headers = [(b'Content-Type', b'text/javascript')]
  body = b"import './echo-cookie-worker.py?key=%s'" % request.GET[b'key']
  return headers, body
