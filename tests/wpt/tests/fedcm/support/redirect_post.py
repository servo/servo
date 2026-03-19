html = """<!DOCTYPE html>
<script>
opener.postMessage("%s", "*")
</script>
"""

def main(request, response):
  host = request.headers.get(b"host")
  expected_referer = b"https://%s/fedcm/support/intercept_header.py" % (host)

  result = "PASS"
  if request.method != "POST":
    result = "FAIL, wrong method"
  elif request.cookies.get(b"same_site_strict") == b"1":
    result = "FAIL, should not send SameSite=Strict cookies, got <%s>" % (request.cookies)
  elif request.cookies.get(b"same_site_lax") == b"1":
    result = "FAIL, should not send SameSite=Lax cookies for POST, got <%s>" % (request.cookies)
  elif request.cookies.get(b"cookie") != b"1":
    result = "FAIL, should send SameSite=None cookies, got <%s>" % (request.cookies)
  elif request.headers.get(b"referer") != expected_referer:
    result = "FAIL, wrong referer: <%s>" % (request.headers.get(b"referer"))
  elif request.headers.get(b"content-type") != b"application/x-www-form-urlencoded":
    result = "FAIL, wrong content type header: <%s>" % (request.headers.get(b"content-type"))
  elif request.body != b"redirect_post":
    result = "FAIL, wrong body: <%s>" % (request.body)
  return html % (result)
