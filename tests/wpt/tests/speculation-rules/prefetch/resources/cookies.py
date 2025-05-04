import json

# TODO(https://crbug.com/406819294): Simplify relative import for util.
import importlib
util = importlib.import_module("speculation-rules.prefetch.resources.util")

def main(request, response):
  cookies = json.dumps({
      key.decode("utf-8"): request.cookies[key].value.decode("utf-8")
      for key in request.cookies
  })

  sec_purpose = request.headers.get("Sec-Purpose", b"").decode("utf-8")

  cookie_count = int(
      request.cookies[b"count"].value) if b"count" in request.cookies else 0
  response.set_cookie("count", f"{cookie_count+1}",
                      secure=True, samesite="None")
  response.set_cookie(
      "type", "prefetch" if sec_purpose.startswith("prefetch") else "navigate")

  headers = [(b"Content-Type", b"text/html"), (b"Cache-Control", b"no-store")]

  if b"cookieindices" in request.GET:
    headers.extend([(b"Vary", b"Cookie"), (b"Cookie-Indices", b"\"vary1\", \"vary2\"")])

  content = util.get_executor_html(
    request,
    f'window.requestCookies = {cookies};')

  return headers, content
