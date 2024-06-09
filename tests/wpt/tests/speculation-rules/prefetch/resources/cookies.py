import json

def main(request, response):
  cookies = json.dumps({
      key.decode("utf-8"): request.cookies[key].value.decode("utf-8")
      for key in request.cookies
  })

  purpose = request.headers.get("Purpose", b"").decode("utf-8")
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

  content = f'''
  <!DOCTYPE html>
  <script src="/common/dispatcher/dispatcher.js"></script>
  <script src="utils.sub.js"></script>
  <script>
  window.requestHeaders = {{
    purpose: "{purpose}",
    sec_purpose: "{sec_purpose}"
  }};

  window.requestCookies = {cookies};

  const uuid = new URLSearchParams(location.search).get('uuid');
  window.executor = new Executor(uuid);
  </script>
  '''
  return headers, content
