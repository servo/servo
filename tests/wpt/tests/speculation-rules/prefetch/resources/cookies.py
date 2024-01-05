
def main(request, response):
  def get_cookie(key):
    key = key.encode("utf-8")
    if key in request.cookies:
      return f'"{request.cookies[key].value.decode("utf-8")}"'
    else:
      return "undefined"

  purpose = request.headers.get("Purpose", b"").decode("utf-8")
  sec_purpose = request.headers.get("Sec-Purpose", b"").decode("utf-8")

  cookie_count = int(
      request.cookies[b"count"].value) if b"count" in request.cookies else 0
  response.set_cookie("count", f"{cookie_count+1}",
                      secure=True, samesite="None")
  response.set_cookie(
      "type", "prefetch" if sec_purpose.startswith("prefetch") else "navigate")

  headers = [(b"Content-Type", b"text/html"), (b"Cache-Control", b"no-store")]

  content = f'''
  <!DOCTYPE html>
  <script src="/common/dispatcher/dispatcher.js"></script>
  <script src="utils.sub.js"></script>
  <script>
  window.requestHeaders = {{
    purpose: "{purpose}",
    sec_purpose: "{sec_purpose}"
  }};

  window.requestCookies = {{
    count: {get_cookie("count")},
    type:  {get_cookie("type")}
  }};

  const uuid = new URLSearchParams(location.search).get('uuid');
  window.executor = new Executor(uuid);
  </script>
  '''
  return headers, content
