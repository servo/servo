import os
import uuid
import hashlib
import time
import json


def bytes_to_strings(d):
  # Recursively convert bytes to strings in `d`.
  if not isinstance(d, dict):
    if isinstance(d, (tuple,list,set)):
      v = [bytes_to_strings(x) for x in d]
      return v
    else:
      if isinstance(d, bytes):
        d = d.decode()
      return d

  result = {}
  for k,v in d.items():
    if isinstance(k, bytes):
      k = k.decode()
    if isinstance(v, dict):
      v = bytes_to_strings(v)
    elif isinstance(v, (tuple,list,set)):
      v = [bytes_to_strings(x) for x in v]
    elif isinstance(v, bytes):
      v = v.decode()
    result[k] = v
  return result


def main(request, response):
  # This condition avoids false positives from CORS preflight checks, where the
  # request under test may be followed immediately by a request to the same URL
  # using a different HTTP method.
  if b'requireOPTIONS' in request.GET and request.method != b'OPTIONS':
      return

  if b'key' in request.GET:
    key = request.GET[b'key']
  elif b'key' in request.POST:
    key = request.POST[b'key']

  ## Convert the key from String to UUID valid String ##
  testId = hashlib.md5(key).hexdigest()

  ## Handle the header retrieval request ##
  if b'retrieve' in request.GET:
    recorded_headers = request.server.stash.take(testId)

    if recorded_headers is None:
      return (204, [], b'')

    return (200, [], recorded_headers)

  ## Record incoming fetch metadata header value
  else:
    try:
      request.server.stash.put(testId, json.dumps(bytes_to_strings(request.headers)))
    except KeyError:
      ## The header is already recorded or it doesn't exist
      pass

    ## Prevent the browser from caching returned responses and allow CORS ##
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Cache-Control", b"no-cache, no-store, must-revalidate")
    response.headers.set(b"Pragma", b"no-cache")
    response.headers.set(b"Expires", b"0")
    if b"mime" in request.GET:
        response.headers.set(b"Content-Type", request.GET.first(b"mime"))

    return request.GET.first(b"body", request.POST.first(b"body", b""))
