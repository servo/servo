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

  return (status, [("Content-Type", "text/html")], f"""
<!DOCTYPE HTML>
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
