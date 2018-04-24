# This serves the worker JavaScript file. It takes a |greeting| request
# parameter to inject into the JavaScript to indicate how the request
# reached the server.
import os
import sys

def main(request, response):
  path = os.path.join(os.path.dirname(__file__),
                      "worker-interception-redirect-webworker.js")
  body = open(path, "rb").read()
  if "greeting" in request.GET:
    body = body.replace("%GREETING_TEXT%", request.GET["greeting"])
  else:
    body = body.replace("%GREETING_TEXT%", "")

  headers = []
  headers.append(("Content-Type", "text/javascript"))

  return headers, body
