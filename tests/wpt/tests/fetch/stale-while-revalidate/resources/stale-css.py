def main(request, response):

    token = request.GET.first(b"token", None)
    is_query = request.GET.first(b"query", None) != None
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
      headers = [(b"Count", count)]
      content = b""
      return 200, headers, content
    else:
      content = b"body { background: rgb(0, 128, 0); }"
      if count > 1:
        content = b"body { background: rgb(255, 0, 0); }"

      headers = [(b"Content-Type", b"text/css"),
                 (b"Cache-Control", b"private, max-age=0, stale-while-revalidate=60")]

      return 200, headers, content
