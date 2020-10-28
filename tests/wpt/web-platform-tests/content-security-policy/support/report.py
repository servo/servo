import time
import json
import re

from wptserve.utils import isomorphic_decode

def retrieve_from_stash(request, key, timeout, default_value):
  t0 = time.time()
  while time.time() - t0 < timeout:
    time.sleep(0.5)
    value = request.server.stash.take(key=key)
    if value is not None:
      return value

  return default_value

def main(request, response):
  op = request.GET.first(b"op")
  key = request.GET.first(b"reportID")
  cookie_key = re.sub(b'^....', b'cccc', key)
  count_key = re.sub(b'^....', b'dddd', key)

  try:
    timeout = request.GET.first(b"timeout")
  except:
    timeout = 0.5
  timeout = float(timeout)

  if op == b"retrieve_report":
    return [(b"Content-Type", b"application/json")], retrieve_from_stash(request, key, timeout, json.dumps({u'error': u'No such report.', u'guid' : isomorphic_decode(key)}))

  if op == b"retrieve_cookies":
    return [(b"Content-Type", b"application/json")], u"{ \"reportCookies\" : " + str(retrieve_from_stash(request, cookie_key, timeout, u"\"None\"")) + u"}"

  if op == b"retrieve_count":
    return [(b"Content-Type", b"application/json")], json.dumps({u'report_count': str(retrieve_from_stash(request, count_key, timeout, 0))})

  # save cookies
  if len(request.cookies.keys()) > 0:
    # convert everything into strings and dump it into a dict so it can be jsoned
    temp_cookies_dict = {}
    for dict_key in request.cookies.keys():
      temp_cookies_dict[isomorphic_decode(dict_key)] = str(request.cookies.get_list(dict_key))
    with request.server.stash.lock:
      request.server.stash.take(key=cookie_key)
      request.server.stash.put(key=cookie_key, value=json.dumps(temp_cookies_dict))

  # save latest report
  report = request.body
  report.rstrip()
  with request.server.stash.lock:
    request.server.stash.take(key=key)
    request.server.stash.put(key=key, value=report)

  with request.server.stash.lock:
    # increment report count
    count = request.server.stash.take(key=count_key)
    if count is None:
      count = 0
    count += 1
    request.server.stash.put(key=count_key, value=count)

  # return acknowledgement report
  return [(b"Content-Type", b"text/plain")], b"Recorded report " + report
