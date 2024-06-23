def main(request, response):
  query_string = request.url_parts[3]
  # We mark the cookie as HttpOnly so that this request
  # can be made before login.html, which would overwrite
  # the value to 1.
  header_value = "accounts={}; SameSite=None; Secure; HttpOnly".format(query_string)
  response.headers.set(b"Set-Cookie", header_value.encode("utf-8"))
  response.headers.set(b"Content-Type", b"text/html")

  return """
<!DOCTYPE html>
<script>
// The important part of this page are the headers.

// If this page was opened as a popup, notify the opener.
if (window.opener) {
  window.opener.postMessage("done_loading", "*");
  window.close();
}
</script>
Sent header value: {}".format(header_value)
"""
