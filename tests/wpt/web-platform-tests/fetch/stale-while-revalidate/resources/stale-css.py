def main(request, response):

    token = request.GET.first("token", None)
    is_query = request.GET.first("query", None) != None
    with request.server.stash.lock:
      value = request.server.stash.take(token)
      count = 0
      if value != None:
        count = int(value)
      if is_query:
        if count < 2:
          request.server.stash.put(token, count)
      else:
        count = count + 1
        request.server.stash.put(token, count)
    if is_query:
      headers = [("Count", count)]
      content = ""
      return 200, headers, content
    else:
      content = "body { background: rgb(0, 128, 0); }"
      if count > 1:
        content = "body { background: rgb(255, 0, 0); }"

      headers = [("Content-Type", "text/css"),
               ("Cache-Control", "private, max-age=0, stale-while-revalidate=60")]

      return 200, headers, content
