
def main(request, response):
  def fmt(x):
    return f'"{x.decode("utf-8")}"' if x is not None else "undefined"

  purpose = request.headers.get("Purpose", b"").decode("utf-8")
  sec_purpose = request.headers.get("Sec-Purpose", b"").decode("utf-8")

  headers = [
    (b"Content-Type", b"text/html"),
    (b'WWW-Authenticate', b'Basic'),
    (b'Cache-Control', b'no-store')
  ]
  status = 200 if request.auth.username is not None or sec_purpose.startswith(
      "prefetch") else 401

  content = f'''
  <!DOCTYPE html>
  <script src="/common/dispatcher/dispatcher.js"></script>
  <script src="utils.sub.js"></script>
  <script>
  window.requestHeaders = {{
    purpose: "{purpose}",
    sec_purpose: "{sec_purpose}"
  }};

  window.requestCredentials = {{
    username: {fmt(request.auth.username)},
    password: {fmt(request.auth.password)}
  }};

  const uuid = new URLSearchParams(location.search).get('uuid');
  window.executor = new Executor(uuid);
  </script>
  '''
  return status, headers, content
