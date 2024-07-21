def main(request, response):
  script = request.GET.first(b"script")

  # Some, but not all, urls will send a query parameter indicating their
  # script will want to postMessage the parent to ack that it has loaded.
  should_ack_load = b"false"
  try:
    # The call to request.GET.first will fail if the parameter isn't present,
    # that's ok.
    if request.GET.first(b"should_ack_load") == b"true":
      should_ack_load = b"true"
  except:
    pass
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
    var should_ack_load = %s;
  </script>

  <body>
  <script src="%s"></script>
  </body>

  """ % (cookie_header, should_ack_load, script)

  return (200, [], body)
