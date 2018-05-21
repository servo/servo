import os.path

def main(request, response):
  header = [('Content-Type', 'text/html')]
  if 'test' in request.GET:
    with open(os.path.join(os.path.dirname(__file__),'blank.html'), 'r') as f:
      body = f.read()
    return (header, body)

  if 'sandbox' in request.GET:
    header.append(('Content-Security-Policy',
                   'sandbox %s' % request.GET['sandbox']))
  with open(os.path.join(os.path.dirname(__file__),
                         'sandboxed-iframe-fetch-event-iframe.html'), 'r') as f:
    body = f.read()
  return (header, body)
