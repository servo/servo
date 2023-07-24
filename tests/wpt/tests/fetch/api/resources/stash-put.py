from wptserve.utils import isomorphic_decode

def main(request, response):
  if request.method == u'OPTIONS':
    # CORS preflight
    response.headers.set(b'Access-Control-Allow-Origin', b'*')
    response.headers.set(b'Access-Control-Allow-Methods', b'*')
    response.headers.set(b'Access-Control-Allow-Headers', b'*')
    return 'done'

  url_dir = u'/'.join(request.url_parts.path.split(u'/')[:-1]) + u'/'
  key = request.GET.first(b'key')
  value = request.GET.first(b'value')
  # value here must be a text string. It will be json.dump()'ed in stash-take.py.
  request.server.stash.put(key, isomorphic_decode(value), url_dir)

  if b'disallow_origin' not in request.GET:
    response.headers.set(b'Access-Control-Allow-Origin', b'*')
  return 'done'
