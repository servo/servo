def main(request, response):
  response.headers.set(b"Access-Control-Allow-Origin", b"*")
  response.headers.set(b"Access-Control-Max-Age", b"0")
  response.headers.set(b"Timing-Allow-Origin", b"*")
  # If this script is accessed with the header X-Require-Preflight then the
  # browser will send a preflight request. Otherwise it won't.
  if request.method == u'OPTIONS':
    response.headers.set(b"Access-Control-Allow-Headers",
        b"X-Require-Preflight")
