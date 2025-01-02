# This serves the worker JavaScript file. It takes a |greeting| request
# parameter to inject into the JavaScript to indicate how the request
# reached the server.
import os

from wptserve.utils import isomorphic_decode

def main(request, response):
  path = os.path.join(os.path.dirname(isomorphic_decode(__file__)),
                      u"worker-interception-redirect-webworker.js")
  body = open(path, u"rb").read()
  if b"greeting" in request.GET:
    body = body.replace(b"%GREETING_TEXT%", request.GET[b"greeting"])
  else:
    body = body.replace(b"%GREETING_TEXT%", b"")

  headers = []
  headers.append((b"Content-Type", b"text/javascript"))

  return headers, body
