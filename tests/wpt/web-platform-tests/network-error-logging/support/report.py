import time
import json
import re

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
  if request.method == 'OPTIONS':
    # Always reject preflights for one subdomain
    if "www2" in request.headers["Origin"]:
      return (400, [], "CORS preflight rejected for www2")
    return [
      ("Content-Type", "text/plain"),
      ("Access-Control-Allow-Origin", "*"),
      ("Access-Control-Allow-Methods", "post"),
      ("Access-Control-Allow-Headers", "Content-Type"),
    ], "CORS allowed"

  op = request.GET.first("op");
  key = request.GET.first("reportID")

  if op == "retrieve_report":
    try:
      timeout = float(request.GET.first("timeout"))
    except:
      timeout = 0.5
    try:
      min_count = int(request.GET.first("min_count"))
    except:
      min_count = 1
    return [("Content-Type", "application/json")], retrieve_from_stash(request, key, timeout, min_count, '[]')

  # append new reports
  new_reports = json.loads(request.body)
  for report in new_reports:
    report["metadata"] = {
      "content_type": request.headers["Content-Type"],
    }
  with request.server.stash.lock:
    reports = request.server.stash.take(key=key)
    if reports is None:
      reports = []
    reports.extend(new_reports)
    request.server.stash.put(key=key, value=reports)

  # return acknowledgement report
  return [("Content-Type", "text/plain")], "Recorded report"
