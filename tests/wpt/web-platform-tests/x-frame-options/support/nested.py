def main(request, response):
    origin = request.GET.first("origin");
    value = request.GET.first("value");
    return ([("Content-Type", "text/html")],
            """<!DOCTYPE html>
<title>XFO.</title>
<body>
<script>
  var gotMessage = false;
  window.addEventListener("message", e => {
    gotMessage = true;
    window.parent.postMessage(e.data, "*");
  });

  var i = document.createElement("iframe");
  i.src = "%s/x-frame-options/support/xfo.py?value=%s";
  i.onload = _ => {
    // Why two rAFs? Because that seems to be enough to stop the
    // load event from racing with the onmessage event.
    requestAnimationFrame(_ => {
      requestAnimationFrame(_ => {
        if (!gotMessage) {
          window.parent.postMessage("Failed", "*");
        }
      });
    });
  };
  document.body.appendChild(i);
</script>
            """ % (origin, value))

