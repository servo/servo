import json

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

  return (status, [("Content-Type", "text/html")], f"""
<!DOCTYPE HTML>
<script src="/common/dispatcher/dispatcher.js"></script>
<script src="./executor-common.js"></script>
<script src="./executor-window.js"></script>

<body>
<script>
window.__requestHeaders = new Headers();
{initRequestHeaders}
requestExecutor();
</script>
""")
