def main(request, response):
  new_url = request.url.replace("redirect", "prefetch").encode("utf-8")
  return 301, [(b"Location", new_url)], b""
