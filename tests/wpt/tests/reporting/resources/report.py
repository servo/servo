import time
import json
import re
import uuid

from wptserve.utils import isomorphic_decode


def retrieve_from_stash(request, key, timeout, default_value, min_count=None, retain=False):
  """Retrieve the set of reports for a given report ID.

  This will extract either the set of reports, credentials, or request count
  from the stash (depending on the key passed in) and return it encoded as JSON.

  When retrieving reports, this will not return any reports until min_count
  reports have been received.

  If timeout seconds elapse before the requested data can be found in the stash,
  or before at least min_count reports are received, default_value will be
  returned instead."""
  t0 = time.time()
  while time.time() - t0 < timeout:
    time.sleep(0.5)
    with request.server.stash.lock:
      value = request.server.stash.take(key=key)
      if value is not None:
        have_sufficient_reports = (
          min_count is None or len(value) >= min_count)
        if retain or not have_sufficient_reports:
          request.server.stash.put(key=key, value=value)
        if have_sufficient_reports:
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

  # Delete reports as requested
  if request.method == u'POST':
    body = json.loads(request.body)
    if (isinstance(body, dict) and "op" in body):
      if body["op"] == "DELETE" and "reportIDs" in body:
        with request.server.stash.lock:
          for key in body["reportIDs"]:
            request.server.stash.take(key=key)
        return "reports cleared"
      response.status = 400
      return "op parameter value not recognized"

  if b"reportID" in request.GET:
    key = request.GET.first(b"reportID")
  elif b"endpoint" in request.GET:
    key = uuid.uuid5(uuid.NAMESPACE_OID, isomorphic_decode(
      request.GET[b'endpoint'])).urn.encode('ascii')[9:]
  else:
    response.status = 400
    return "Either reportID or endpoint parameter is required."

  # Cookie and count keys are derived from the report ID.
  cookie_key = re.sub(b'^....', b'cccc', key)
  count_key = re.sub(b'^....', b'dddd', key)

  if request.method == u'GET':
    try:
      timeout = float(request.GET.first(b"timeout"))
    except:
      timeout = 0.5
    try:
      min_count = int(request.GET.first(b"min_count"))
    except:
      min_count = 1
    retain = (b"retain" in request.GET)

    op = request.GET.first(b"op", b"")
    if op in (b"retrieve_report", b""):
      return [(b"Content-Type", b"application/json")], retrieve_from_stash(request, key, timeout, u'[]', min_count, retain)

    if op == b"retrieve_cookies":
      return [(b"Content-Type", b"application/json")], u"{ \"reportCookies\" : " + str(retrieve_from_stash(request, cookie_key, timeout, u"\"None\"")) + u"}"

    if op == b"retrieve_count":
      return [(b"Content-Type", b"application/json")], u"{ \"report_count\": %s }" % retrieve_from_stash(request, count_key, timeout, 0)

    response.status = 400
    return "op parameter value not recognized."

  # Save cookies.
  if len(request.cookies.keys()) > 0:
    # Convert everything into strings and dump it into a dict.
    temp_cookies_dict = {}
    for dict_key in request.cookies.keys():
      temp_cookies_dict[isomorphic_decode(dict_key)] = str(
        request.cookies.get_list(dict_key))
    with request.server.stash.lock:
      # Clear any existing cookie data for this request before storing new data.
      request.server.stash.take(key=cookie_key)
      request.server.stash.put(key=cookie_key, value=temp_cookies_dict)

  # Append new report(s).
  new_reports = json.loads(request.body)

  # If the incoming report is a CSP report-uri report, then it will be a single
  # dictionary rather than a list of reports. To handle this case, ensure that
  # any non-list request bodies are wrapped in a list.
  if not isinstance(new_reports, list):
    new_reports = [new_reports]

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

  # Increment report submission count. This tracks the number of times this
  # reporting endpoint was contacted, rather than the total number of reports
  # submitted, which can be seen from the length of the report list.
  with request.server.stash.lock:
    count = request.server.stash.take(key=count_key)
    if count is None:
      count = 0
    count += 1
    request.server.stash.put(key=count_key, value=count)

  # Return acknowledgement report.
  return [(b"Content-Type", b"text/plain")], b"Recorded report " + request.body
