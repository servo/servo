def main(request, response):
  script = request.GET.first(b"script")
  cookie_header = request.headers.get(b"Cookie", b"")

  body = b"""
  <!DOCTYPE html>
  <meta charset="utf-8">
  <title>Subframe with HTTP Cookies</title>
  <script src="/resources/testharness.js"></script>
  <script src="/resources/testdriver.js"></script>
  <script src="/resources/testdriver-vendor.js"></script>
  <script>
    var httpCookies = "%s";
  </script>

  <script src="%s"></script>
  """ % (cookie_header, script)

  return (200, [], body)
