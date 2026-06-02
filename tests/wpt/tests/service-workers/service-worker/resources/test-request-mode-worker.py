import json
import os
import uuid
import sys

from wptserve.utils import isomorphic_decode

def main(request, response):
  path = os.path.join(os.path.dirname(isomorphic_decode(__file__)),
                      u"test-request-mode-worker.js")
  body = open(path, u"rb").read()

  data = {isomorphic_decode(key):isomorphic_decode(request.headers[key]) for key, value in request.headers.items()}

  body = body.replace(b"%HEADERS%", json.dumps(data).encode("utf-8"))
  body = body.replace(b"%UUID%", str(uuid.uuid4()).encode("utf-8"))

  headers = []
  headers.append((b"ETag", b"etag"))
  headers.append((b"Content-Type", b'text/javascript'))

  return headers, body
