import time

def main(request, response):
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

  content = (f'<!DOCTYPE html>\n'
             f'<script src="/common/dispatcher/dispatcher.js"></script>\n'
             f'<script src="utils.sub.js"></script>\n'
             f'<script>\n'
             f'  window.requestHeaders = {{\n'
             f'    purpose: "{request.headers.get("Purpose", b"").decode("utf-8")}",\n'
             f'    sec_purpose: "{request.headers.get("Sec-Purpose", b"").decode("utf-8")}",\n'
             f'    referer: "{request.headers.get("Referer", b"").decode("utf-8")}",\n'
             f'  }};\n'
             f'  const uuid = new URLSearchParams(location.search).get("uuid");\n'
             f'  window.executor = new Executor(uuid);\n'
             f'</script>\n')

  return content
