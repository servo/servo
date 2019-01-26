import os.path

def main(request, response):

    cookie = request.cookies.first("Count", None)
    count = 0
    if cookie != None:
      count = int(cookie.value)
    if request.GET.first("query", None) != None:
      headers = [("Count", count)]
      content = ""
      return 200, headers, content
    else:
      count = count + 1
      filename = "green-16x16.png"
      if cookie > 1:
        filename = "green-256x256.png"

      path = os.path.join(os.path.dirname(__file__), "../../images", filename)
      body = open(path, "rb").read()

      response.add_required_headers = False
      response.writer.write_status(200)
      response.writer.write_header("content-length", len(body))
      response.writer.write_header("Cache-Control", "private, max-age=0, stale-while-revalidate=10")
      response.writer.write_header("content-type", "image/png")
      response.writer.write_header("Set-Cookie", "Count={}".format(count))
      response.writer.end_headers()

      response.writer.write(body)
