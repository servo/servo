import base64

def maybeBoolToJavascriptLiteral(value):
  if value == None:
    return "undefined"
  if value == True:
    return "true"
  if value == False:
    return "false"
  raise ValueError("Expected bool or None")

def main(request, response):
  destination = request.headers.get("sec-fetch-dest").decode("utf-8")
  gpcValue = request.headers.get("sec-gpc") == b'1'
  expectedGPCValue = request.GET.get(b"gpc") == b"true"
  inFrame = request.GET.get(b"framed") != None
  destinationDescription = "framed " + destination if inFrame else destination
  if destination == "document" or destination == "iframe":
    response.headers.set('Content-Type', 'text/html');
    return f"""
<!DOCTYPE html>
<html>
<title>Sec-GPC {destination}</title>
<head>
  <script src="/resources/testharness.js"></script>
</head>
<body>
  <div id="log"></div>
  <img id="imageTest">
  <script>
    test(function(t) {{
      assert_equals({maybeBoolToJavascriptLiteral(gpcValue)}, {maybeBoolToJavascriptLiteral(expectedGPCValue)}, "Expected Sec-GPC value ({maybeBoolToJavascriptLiteral(expectedGPCValue)}) is on the {destinationDescription} fetch");
    }}, `Expected Sec-GPC value ({maybeBoolToJavascriptLiteral(expectedGPCValue)}) is on the {destinationDescription} fetch`);
    promise_test(function(t) {{
      const image = document.getElementById("imageTest");
      const testResult = new Promise((resolve, reject) => {{
        image.addEventListener('load', resolve);
        image.addEventListener('error', reject);
      }});
      image.src = "getGPC.py?gpc={maybeBoolToJavascriptLiteral(expectedGPCValue)}";
      return testResult;
    }}, `Expected Sec-GPC value ({maybeBoolToJavascriptLiteral(expectedGPCValue)}) is on the {"framed " if destination == "iframe" or inFrame else ""}image fetch`);
  </script>
""" + (f"""
  <script>
    const iframe = document.createElement("iframe");
    iframe.src = "getGPC.py?gpc={maybeBoolToJavascriptLiteral(expectedGPCValue)}";
    document.body.appendChild(iframe);
    async function run() {{
      await Promise.all([
        fetch_tests_from_window(iframe.contentWindow),
        fetch_tests_from_worker(new Worker("getGPC.py?gpc={maybeBoolToJavascriptLiteral(expectedGPCValue)}")),
        fetch_tests_from_worker(new SharedWorker("getGPC.py?gpc={maybeBoolToJavascriptLiteral(expectedGPCValue)}")),
      ]);
      let r = await navigator.serviceWorker.register(
        "getGPC.py?gpc={maybeBoolToJavascriptLiteral(expectedGPCValue)}",
        {{scope: "./blank.html"}});
      let sw = r.active || r.installing || r.waiting;
      await fetch_tests_from_worker(sw);
      await r.unregister();
    }}
    run();
  </script>
  """ if destination == "document" else "") + f"""
  <script src="getGPC.py?gpc={maybeBoolToJavascriptLiteral(expectedGPCValue)}{"&framed" if destination == "iframe" or inFrame else ""}"></script>
</body>
</html>
"""
  elif destination == "image":
    if gpcValue == expectedGPCValue:
      return (200, [(b"Content-Type", b"image/png")], base64.b64decode("iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAEUlEQVR42mP8nzaTAQQYYQwALssD/5ca+r8AAAAASUVORK5CYII="))
    return (400, [], "")
  elif destination == "script":
    response.headers.set('Content-Type', 'application/javascript');
    return f"""
debugger;
test(function(t) {{
  assert_equals({maybeBoolToJavascriptLiteral(gpcValue)}, {maybeBoolToJavascriptLiteral(expectedGPCValue)}, "Expected Sec-GPC value ({maybeBoolToJavascriptLiteral(expectedGPCValue)}) is on the {destinationDescription} fetch");
}}, `Expected Sec-GPC value ({maybeBoolToJavascriptLiteral(expectedGPCValue)}) is on the {destinationDescription} fetch`);
"""
  elif destination == "worker" or destination == "sharedworker" or destination == "serviceworker":
    response.headers.set('Content-Type', 'application/javascript');
    return f"""
importScripts("/resources/testharness.js");
test(function(t) {{
  assert_equals({maybeBoolToJavascriptLiteral(gpcValue)}, {maybeBoolToJavascriptLiteral(expectedGPCValue)}, "Expected Sec-GPC value ({maybeBoolToJavascriptLiteral(expectedGPCValue)}) is on the {destinationDescription} fetch");
}}, `Expected Sec-GPC value ({maybeBoolToJavascriptLiteral(expectedGPCValue)}) is on the {destinationDescription} fetch`);
done();
"""
  raise ValueError("Unexpected destination")
