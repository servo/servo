def main(request, response):

    token = request.GET.first("token", None)
    value = request.server.stash.take(token)
    count = 0
    if value != None:
      count = int(value)
    if request.GET.first("query", None) != None:
      headers = [("Count", count)]
      content = ""
      if count < 2:
        request.server.stash.put(token, count)
      return 200, headers, content
    else:
      count = count + 1
      content = "body { background: rgb(0, 128, 0); }"
      if count > 1:
        content = "body { background: rgb(255, 0, 0); }"

      headers = [("Content-Type", "text/css"),
               ("Cache-Control", "private, max-age=0, stale-while-revalidate=60")]

      request.server.stash.put(token, count)
      return 200, headers, content
