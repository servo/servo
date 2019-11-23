import json
import os
import time

def main(request, response):
  path = os.path.join(os.path.dirname(__file__),
                      "test-request-headers-worker.js")
  body = open(path, "rb").read()

  data = {key:request.headers[key] for key,value in request.headers.iteritems()}
  body = body.replace("%HEADERS%", json.dumps(data))
  body = body.replace("%TIMESTAMP%", str(time.time()))

  headers = []
  headers.append(("ETag", "etag"))
  headers.append(("Content-Type", 'text/javascript'))

  return headers, body
