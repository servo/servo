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
      content = "body { background: rgb(0, 128, 0); }"
      if count > 1:
        content = "body { background: rgb(255, 0, 0); }"

      headers = [("Content-Type", "text/css"),
               ("Set-Cookie", "Count={}".format(count)),
               ("Cache-Control", "private, max-age=0, stale-while-revalidate=10")]
      return 200, headers, content
