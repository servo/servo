import html
import json
from urllib import parse

def main(request, response):
  initRequestHeaders = ""
  for header_name in request.headers.keys():
    for header_value in request.headers.get_list(header_name):
      js_name = json.dumps(header_name.lower().decode("utf-8"))
      js_value = json.dumps(header_value.decode("utf-8"))
      initRequestHeaders += f"window.__requestHeaders.append({js_name}, {js_value});\n"
      if (b"status" in request.GET):
            status = int(request.GET.first(b"status"))
      else:
            status = 200
  query = parse.parse_qs(request.url_parts.query)
  scripts = []
  for script in query.get("script", []):
    scripts.append(f"<script src='{html.escape(script)}'></script>")
  scripts_s = "\n".join(scripts)

  uuid = query.get("uuid")[0]

  start_on = query.get("startOn")
  start_on_s = f"'{start_on[0]}'" if start_on else "null"

  headers = [("Content-Type", "text/html")]
  # We always permit partitioned popins to be loaded for testing.
  # See https://explainers-by-googlers.github.io/partitioned-popins/
  if request.headers.get(b"Sec-Popin-Context") == b"partitioned":
    headers.append((b'Popin-Policy', b"partitioned=*"))

  # This sets a base href so that even if this content e.g. data or blob URLs
  # document, relative URLs will resolve.
  return (status, headers, f"""
<!DOCTYPE HTML>
<base href="{html.escape(request.url)}">
<script src="/common/dispatcher/dispatcher.js"></script>
<script src="./executor-common.js"></script>
<script src="./executor-window.js"></script>

{scripts_s}
<body>
<script>
window.__requestHeaders = new Headers();
{initRequestHeaders}
requestExecutor("{uuid}", {start_on_s});
</script>
""")
