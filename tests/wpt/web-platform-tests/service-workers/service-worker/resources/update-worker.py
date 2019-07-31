import urllib

def redirect_response(request, response, visited_count):
  # |visited_count| is used as a unique id to differentiate responses
  # every time.
  location = 'empty.js'
  if 'Redirect' in request.GET:
      location = urllib.unquote(request.GET['Redirect'])
  return (301,
  [
    ('Cache-Control', 'no-cache, must-revalidate'),
    ('Pragma', 'no-cache'),
    ('Content-Type', 'application/javascript'),
    ('Location', location),
  ],
  '/* %s */' % str(visited_count))

def not_found_response():
  return 404, [('Content-Type', 'text/plain')], "Page not found"

def ok_response(request, response, visited_count,
                extra_body='', mime_type='application/javascript'):
  # |visited_count| is used as a unique id to differentiate responses
  # every time.
  return (
    [
      ('Cache-Control', 'no-cache, must-revalidate'),
      ('Pragma', 'no-cache'),
      ('Content-Type', mime_type)
    ],
    '/* %s */ %s' % (str(visited_count), extra_body))

def main(request, response):
  key = request.GET["Key"]
  mode = request.GET["Mode"]

  visited_count = request.server.stash.take(key)
  if visited_count is None:
    visited_count = 0

  # Keep how many times the test requested this resource.
  visited_count += 1
  request.server.stash.put(key, visited_count)

  # Return a response based on |mode| only when it's the second time (== update).
  if visited_count == 2:
    if mode == 'normal':
      return ok_response(request, response, visited_count)
    if mode == 'bad_mime_type':
      return ok_response(request, response, visited_count, mime_type='text/html')
    if mode == 'not_found':
      return not_found_response()
    if mode == 'redirect':
      return redirect_response(request, response, visited_count)
    if mode == 'syntax_error':
      return ok_response(request, response, visited_count, extra_body='badsyntax(isbad;')
    if mode == 'throw_install':
      return ok_response(request, response, visited_count, extra_body="addEventListener('install', function(e) { throw new Error('boom'); });")

  return ok_response(request, response, visited_count)
