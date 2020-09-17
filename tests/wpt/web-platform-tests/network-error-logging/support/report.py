import time
import json
import re

from wptserve.utils import isomorphic_decode

def retrieve_from_stash(request, key, timeout, min_count, default_value):
  t0 = time.time()
  while time.time() - t0 < timeout:
    time.sleep(0.5)
    value = request.server.stash.take(key=key)
    if value is not None and len(value) >= min_count:
      request.server.stash.put(key=key, value=value)
      return json.dumps(value)

  return default_value

def main(request, response):
  # Handle CORS preflight requests
  if request.method == u'OPTIONS':
    # Always reject preflights for one subdomain
    if b"www2" in request.headers[b"Origin"]:
      return (400, [], u"CORS preflight rejected for www2")
    return [
      (b"Content-Type", b"text/plain"),
      (b"Access-Control-Allow-Origin", b"*"),
      (b"Access-Control-Allow-Methods", b"post"),
      (b"Access-Control-Allow-Headers", b"Content-Type"),
    ], u"CORS allowed"

  op = request.GET.first(b"op");
  key = request.GET.first(b"reportID")

  if op == b"retrieve_report":
    try:
      timeout = float(request.GET.first(b"timeout"))
    except:
      timeout = 0.5
    try:
      min_count = int(request.GET.first(b"min_count"))
    except:
      min_count = 1
    return [(b"Content-Type", b"application/json")], retrieve_from_stash(request, key, timeout, min_count, u'[]')

  # append new reports
  new_reports = json.loads(request.body)
  for report in new_reports:
    report[u"metadata"] = {
      u"content_type": isomorphic_decode(request.headers[b"Content-Type"]),
    }
  with request.server.stash.lock:
    reports = request.server.stash.take(key=key)
    if reports is None:
      reports = []
    reports.extend(new_reports)
    request.server.stash.put(key=key, value=reports)

  # return acknowledgement report
  return [(b"Content-Type", b"text/plain")], u"Recorded report"
