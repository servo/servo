import json
import helpers

def main(request, response):
    headers = helpers.setNoCacheAndCORSHeaders(request, response)
    cookies = helpers.readCookies(request)
    headers.append(("Content-Type", "text/html; charset=utf-8"))

    tmpl = """
<!DOCTYPE html>
<script>
  var data = %s;
  data.type = "COOKIES";

  try {
    data.domcookies = document.cookie;
  } catch (e) {}

  if (window.parent != window) {
    window.parent.postMessage(data, "*");
    if (window.top != window.parent)
      window.top.postMessage(data, "*");
  }


  if (window.opener)
    window.opener.postMessage(data, "*");

  window.addEventListener("message", e => {
    console.log(e);
    if (e.data == "reload")
      window.location.reload();
  });
</script>
"""
    return headers, tmpl % json.dumps(cookies)
