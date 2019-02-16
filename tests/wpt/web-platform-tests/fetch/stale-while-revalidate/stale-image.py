import os.path

def main(request, response):

    token = request.GET.first("token", None)
    value = request.server.stash.take(token)
    count = 0
    if value != None:
      count = int(value)
    if request.GET.first("query", None) != None:
      headers = [("Count", count)]
      content = ""
      return 200, headers, content
    else:
      count = count + 1
      filename = "green-16x16.png"
      if count > 1:
        filename = "green-256x256.png"

      path = os.path.join(os.path.dirname(__file__), "../../images", filename)
      body = open(path, "rb").read()

      request.server.stash.put(token, count)
      response.add_required_headers = False
      response.writer.write_status(200)
      response.writer.write_header("content-length", len(body))
      response.writer.write_header("Cache-Control", "private, max-age=0, stale-while-revalidate=60")
      response.writer.write_header("content-type", "image/png")
      response.writer.end_headers()

      response.writer.write(body)
