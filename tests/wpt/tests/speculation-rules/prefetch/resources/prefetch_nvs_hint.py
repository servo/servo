import time

# TODO(https://crbug.com/406819294): Simplify relative import for util.
import importlib
util = importlib.import_module("speculation-rules.prefetch.resources.util")

def main(request, response):
  response.headers.set("Cache-Control", "no-store")
  uuid = request.GET[b"uuid"]
  wait_for_prefetch_start_uuid = None
  if b"wait_for_prefetch_uuid" in request.GET:
    wait_for_prefetch_start_uuid = request.GET[b"wait_for_prefetch_uuid"]
  prefetch = request.headers.get(
      "Sec-Purpose", b"").decode("utf-8").startswith("prefetch")
  if b"unblock" in request.GET:
    request.server.stash.put(uuid, 0)
    return ''

  if b"wait_for_prefetch" in request.GET:
    if wait_for_prefetch_start_uuid is None:
      return ''
    wait_for_prefetch = None
    while wait_for_prefetch is None:
      time.sleep(0.1)
      wait_for_prefetch = request.server.stash.take(wait_for_prefetch_start_uuid)
    return ''

  if b"nvs_header" in request.GET:
    nvs_header = request.GET[b"nvs_header"]
    response.headers.set("No-Vary-Search", nvs_header)

  if prefetch:
    if wait_for_prefetch_start_uuid is not None:
      request.server.stash.put(wait_for_prefetch_start_uuid, 0)
    nvswait = None
    while nvswait is None:
      time.sleep(0.1)
      nvswait = request.server.stash.take(uuid)

  return util.get_executor_html(request, '')
