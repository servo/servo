# This serves a different response to each request, to test service worker
# updates. If |filename| is provided, it writes that file into the body.
#
# Usage:
#   navigator.serviceWorker.register('update_shell.py?filename=worker.js')
#
# This registers worker.js as a service worker, and every update check
# will return a new response.
import os
import random
import time

from wptserve.utils import isomorphic_encode

def main(request, response):
  # Set no-cache to ensure the user agent finds a new version for each update.
  headers = [(b'Cache-Control', b'no-cache, must-revalidate'),
             (b'Pragma', b'no-cache'),
             (b'Content-Type', b'application/javascript')]

  # Return a different script for each access.
  timestamp = u'// %s %s' % (time.time(), random.random())
  body = isomorphic_encode(timestamp) + b'\n'

  # Inject the file into the response.
  if b'filename' in request.GET:
    path = os.path.join(os.path.dirname(isomorphic_encode(__file__)),
                        request.GET[b'filename'])
    with open(path, 'rb') as f:
      body += f.read()

  return headers, body
