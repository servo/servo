import time

def main(request, response):
    token = request.GET["token"]
    response.status = 200
    response.headers.append("Content-Type", "text/html")
    if "verify-token" in request.GET:
      if request.server.stash.take(token):
        return 'TOKEN_SET'
      return 'TOKEN_NOT_SET'

    if "finish-delay" not in request.GET:
      # <a download>
      request.server.stash.put(token, True)
      return

    # navigation to download
    response.headers.append("Content-Disposition", "attachment")
    response.write_status_headers()
    finish_delay = float(request.GET["finish-delay"]) / 1E3
    count = 10
    single_delay = finish_delay / count
    for i in range(count): # pylint: disable=unused-variable
        time.sleep(single_delay)
        response.writer.write_content("\n")
        if not response.writer.flush():
          return
    request.server.stash.put(token, True)
