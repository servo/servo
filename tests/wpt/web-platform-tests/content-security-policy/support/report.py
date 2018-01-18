import time
import json
import re

def retrieve_from_stash(request, key, timeout, default_value):
  t0 = time.time()
  while time.time() - t0 < timeout:
    time.sleep(0.5)
    value = request.server.stash.take(key=key)
    if value is not None:
      return value

  return default_value

def main(request, response):
  op = request.GET.first("op");
  key = request.GET.first("reportID")
  cookie_key = re.sub('^....', 'cccc', key)
  count_key = re.sub('^....', 'dddd', key)

  try:
    timeout = request.GET.first("timeout")
  except:
    timeout = 0.5
  timeout = float(timeout)

  if op == "retrieve_report":
    return [("Content-Type", "application/json")], retrieve_from_stash(request, key, timeout, json.dumps({'error': 'No such report.' , 'guid' : key}))

  if op == "retrieve_cookies":
    return [("Content-Type", "application/json")], "{ \"reportCookies\" : " + str(retrieve_from_stash(request, cookie_key, timeout, "\"None\"")) + "}"

  if op == "retrieve_count":
    return [("Content-Type", "application/json")], json.dumps({'report_count': str(retrieve_from_stash(request, count_key, timeout, 0))})

  # save cookies
  if hasattr(request, 'cookies') and len(request.cookies.keys()) > 0:
    # convert everything into strings and dump it into a dict so it can be jsoned
    temp_cookies_dict = {}
    for dict_key in request.cookies.keys():
      temp_cookies_dict[str(dict_key)] = str(request.cookies.get_list(dict_key))
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
  return [("Content-Type", "text/plain")], "Recorded report " + report
