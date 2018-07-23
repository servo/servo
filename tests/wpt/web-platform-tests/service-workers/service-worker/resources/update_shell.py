# This serves a different response to each request, to test service worker
# updates. If |filename| is provided, it writes that file into the body.
#
# Usage:
#   navigator.serviceWorker.register('update_shell.py?filename=worker.js')
#
# This registers worker.js as a service worker, and every update check
# will return a new response.
import os
import sys
import time

def main(request, response):
  # Set no-cache to ensure the user agent finds a new version for each update.
  headers = [('Cache-Control', 'no-cache, must-revalidate'),
             ('Pragma', 'no-cache'),
             ('Content-Type', 'application/javascript')]

  # Return a different script for each access.  Use .time() and .clock() for
  # best time resolution across different platforms.
  timestamp = '// %s %s' % (time.time(), time.clock())
  body = timestamp + '\n'

  # Inject the file into the response.
  if 'filename' in request.GET:
    path = os.path.join(os.path.dirname(__file__),
                        request.GET['filename'])
    with open(path, 'rb') as f:
      body += f.read()

  return headers, body
