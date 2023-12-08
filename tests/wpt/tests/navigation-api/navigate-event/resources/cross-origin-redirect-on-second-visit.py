import uuid

def redirect_response():
  location = b'http://localhost:8000/common/blank.html'
  return (301,
  [
    (b'Cache-Control', b'no-cache, no-store, must-revalidate'),
    (b'Pragma', b'no-cache'),
    (b'Content-Type', b'text/html'),
    (b'Location', location),
  ],
  b'redirect_body')

def ok_response():
  return (
    [
      (b'Cache-Control', b'no-cache, no-store, must-revalidate'),
      (b'Pragma', b'no-cache'),
      (b'Content-Type', b'text/html')
    ],
    b'body')

def main(request, response):
  key = request.GET[b'key'];
  visited = request.server.stash.take(key)
  request.server.stash.put(key, True)

  if visited is None:
    return ok_response()

  return redirect_response()
