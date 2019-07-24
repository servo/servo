import os

def serve_js_from_file(request, response, filename):
  body = ''
  path = os.path.join(os.path.dirname(__file__), filename)
  with open(path, 'rb') as f:
    body = f.read()
  return (
    [
      ('Cache-Control', 'no-cache, must-revalidate'),
      ('Pragma', 'no-cache'),
      ('Content-Type', 'application/javascript')
    ], body)

def main(request, response):
  key = request.GET["Key"]

  visited_count = request.server.stash.take(key)
  if visited_count is None:
    visited_count = 0

  # Keep how many times the test requested this resource.
  visited_count += 1
  request.server.stash.put(key, visited_count)

  # Serve a file based on how many times it's requested.
  if visited_count == 1:
    return serve_js_from_file(request, response, request.GET["First"])
  if visited_count == 2:
    return serve_js_from_file(request, response, request.GET["Second"])
  raise "Unknown state"
