import os.path

from wptserve.utils import isomorphic_decode

def main(request, response):
  header = [(b'Content-Type', b'text/html')]
  if b'test' in request.GET:
    with open(os.path.join(os.path.dirname(isomorphic_decode(__file__)), u'sample.js'), u'r') as f:
      body = f.read()
    return (header, body)

  if b'sandbox' in request.GET:
    header.append((b'Content-Security-Policy',
                   b'sandbox %s' % request.GET[b'sandbox']))
  with open(os.path.join(os.path.dirname(isomorphic_decode(__file__)),
                         u'sandboxed-iframe-fetch-event-iframe.html'), u'r') as f:
    body = f.read()
  return (header, body)
