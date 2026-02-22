html = """<!DOCTYPE html>
<script>
opener.postMessage("%s", "*")
</script>
"""

def main(request, response):
  result = "PASS"
  if request.method != "POST":
    result = "FAIL, wrong method"
  elif request.headers.get(b"content-type") != b"application/x-www-form-urlencoded":
    result = "FAIL, wrong content type header: <%s>" % (request.headers.get(b"content-type"))
  elif request.body != b"redirect_post":
    result = "FAIL, wrong body: <%s>" % (request.body)
  return html % (result)
