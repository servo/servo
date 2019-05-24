def main(request, response):
    origin = request.GET.first("origin");
    value = request.GET.first("value");
    # This is used to solve the race condition we have for postMessages
    shouldSucceed = request.GET.first("loadShouldSucceed", "false");
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
        // The race condition problem we have is it is possible
        // that the sub iframe is loaded before the postMessage is
        // dispatched, as a result, the "Failed" message is sent
        // out. So the way we fixed is we simply let the timeout
        // to happen if we expect the "Loaded" postMessage to be
        // sent
        if (!gotMessage && %s != true) {
          window.parent.postMessage("Failed", "*");
        }
      });
    });
  };
  document.body.appendChild(i);
</script>
            """ % (origin, value, shouldSucceed))

