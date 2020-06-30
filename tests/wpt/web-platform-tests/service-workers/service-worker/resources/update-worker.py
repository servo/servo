from six.moves.urllib.parse import unquote

from wptserve.utils import isomorphic_decode, isomorphic_encode

def redirect_response(request, response, visited_count):
  # |visited_count| is used as a unique id to differentiate responses
  # every time.
  location = b'empty.js'
  if b'Redirect' in request.GET:
      location = isomorphic_encode(unquote(isomorphic_decode(request.GET[b'Redirect'])))
  return (301,
  [
    (b'Cache-Control', b'no-cache, must-revalidate'),
    (b'Pragma', b'no-cache'),
    (b'Content-Type', b'application/javascript'),
    (b'Location', location),
  ],
  u'/* %s */' % str(visited_count))

def not_found_response():
  return 404, [(b'Content-Type', b'text/plain')], u"Page not found"

def ok_response(request, response, visited_count,
                extra_body=u'', mime_type=b'application/javascript'):
  # |visited_count| is used as a unique id to differentiate responses
  # every time.
  return (
    [
      (b'Cache-Control', b'no-cache, must-revalidate'),
      (b'Pragma', b'no-cache'),
      (b'Content-Type', mime_type)
    ],
    u'/* %s */ %s' % (str(visited_count), extra_body))

def main(request, response):
  key = request.GET[b"Key"]
  mode = request.GET[b"Mode"]

  visited_count = request.server.stash.take(key)
  if visited_count is None:
    visited_count = 0

  # Keep how many times the test requested this resource.
  visited_count += 1
  request.server.stash.put(key, visited_count)

  # Return a response based on |mode| only when it's the second time (== update).
  if visited_count == 2:
    if mode == b'normal':
      return ok_response(request, response, visited_count)
    if mode == b'bad_mime_type':
      return ok_response(request, response, visited_count, mime_type=b'text/html')
    if mode == b'not_found':
      return not_found_response()
    if mode == b'redirect':
          return redirect_response(request, response, visited_count)
    if mode == b'syntax_error':
      return ok_response(request, response, visited_count, extra_body=u'badsyntax(isbad;')
    if mode == b'throw_install':
      return ok_response(request, response, visited_count, extra_body=u"addEventListener('install', function(e) { throw new Error('boom'); });")

  return ok_response(request, response, visited_count)
