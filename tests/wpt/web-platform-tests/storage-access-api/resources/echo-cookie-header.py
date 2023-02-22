def main(request, response):
  cookie_header = request.headers.get(b"Cookie", b"")

  return (200, [], cookie_header)
