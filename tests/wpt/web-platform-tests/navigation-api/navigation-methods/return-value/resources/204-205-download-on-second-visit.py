def main(request, response):
    key = request.GET[b"id"]

    # If hit with a POST with ?action=X, store X in the stash
    if request.method == "POST":
      action = request.GET[b"action"]
      request.server.stash.put(key, action)

      return (204, [], "")

    # If hit with a GET, either return a normal initial page, or the abnormal requested response
    elif request.method == "GET":
      action = request.server.stash.take(key)

      if action is None:
          return (200, [("Content-Type", "text/html"), ("Cache-Control", "no-store")], "initial page")
      if action == b"204":
          return (204, [], "")
      if action == b"205":
          return (205, [], "")
      if action == b"download":
          return (200, [("Content-Type", "text/plain"), ("Content-Disposition", "attachment")], "some text to download")

    return (400, [], "")
