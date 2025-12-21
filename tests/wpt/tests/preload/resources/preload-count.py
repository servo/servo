import sys

def main(request, response):
  key = b"a8697ae7-c8cb-4dbd-a8ef-27111dc7042f"
  count = request.server.stash.take(key)
  if count is None:
    count = 0

  action = request.GET.first(b"action")
  if action == b"result":
    response.headers.append(b"Content-Type", b"text/javascript")
    return "preloadCount = {0};".format(count)
  else:
    count += 1
    request.server.stash.put(key, count)
    response.status = 404
    return 'No entry is found'
