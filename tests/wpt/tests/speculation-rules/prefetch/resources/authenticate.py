# TODO(https://crbug.com/406819294): Simplify relative import for util.
import importlib
util = importlib.import_module("speculation-rules.prefetch.resources.util")

def main(request, response):
  def fmt(x):
    return f'"{x.decode("utf-8")}"' if x is not None else "undefined"

  sec_purpose = request.headers.get("Sec-Purpose", b"").decode("utf-8")

  headers = [
    (b"Content-Type", b"text/html"),
    (b'WWW-Authenticate', b'Basic'),
    (b'Cache-Control', b'no-store')
  ]
  status = 200 if request.auth.username is not None or sec_purpose.startswith(
      "prefetch") else 401

  content = util.get_executor_html(
    request,
    f'''window.requestCredentials = {{
        username: {fmt(request.auth.username)},
        password: {fmt(request.auth.password)}
      }};
    ''')

  return status, headers, content
