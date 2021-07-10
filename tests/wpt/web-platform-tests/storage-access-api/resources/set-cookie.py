def main(request, response):
  name = request.GET.first(b"name")
  value = request.GET.first(b"value")
  testcase = request.GET.first(b"testcase")
  response_headers = [(b"Set-Cookie", name + b"=" + value)]

  body = b"""
  <!DOCTYPE html>
  <meta charset="utf-8">
  <title>Set Storage Access Subframe</title>
  <script src="/resources/testharness.js"></script>

  <script>
    let querystring = window.location.search.substring(1).split("&");
    const allowed = querystring.some(param => param.toLowerCase() === "allowed=true");

    test(() => {
      if (allowed) {
        assert_equals(document.cookie, "%s=%s");
      } else {
        assert_equals(document.cookie, "");
      }
    }, "[%s] Cookie access is allowed: " + allowed);
  </script>
  """ % (name, value, testcase)

  return (200, response_headers, body)
