def main(request, response):
    coop = request.GET.first("coop")
    coep = request.GET.first("coep")
    redirect = request.GET.first("redirect", None)
    if coop != "":
        response.headers.set("Cross-Origin-Opener-Policy", coop)
    if coep != "":
        response.headers.set("Cross-Origin-Embedder-Policy", coep)
    if 'cache' in request.GET:
        response.headers.set('Cache-Control', 'max-age=3600')

    if redirect != None:
        response.status = 302
        response.headers.set("Location", redirect)
        return

    # This uses an <iframe> as BroadcastChannel is same-origin bound.
    response.content = """
<!doctype html>
<meta charset=utf-8>
<script src="/common/get-host-info.sub.js"></script>
<body></body>
<script>
  const params = new URL(location).searchParams;
  const navHistory = params.get("navHistory");
  const avoidBackAndForth = params.get("avoidBackAndForth");
  const navigate = params.get("navigate");
  // Need to wait until the page is fully loaded before navigating
  // so that it creates a history entry properly.
  const fullyLoaded = new Promise((resolve, reject) => {
    addEventListener('load', () => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          resolve();
        });
      });
    });
  });
  if (navHistory !== null) {
    fullyLoaded.then(() => {
      history.go(Number(navHistory));
    });
  } else if (navigate !== null && (history.length === 1 || !avoidBackAndForth)) {
    fullyLoaded.then(() => {
      self.location = navigate;
    });
  } else {
    let openerDOMAccessAllowed = false;
    try {
      openerDOMAccessAllowed = !!self.opener.document.URL;
    } catch(ex) {
    }
    // Handle the response from the frame, closing the popup once the
    // test completes.
    addEventListener("message", event => {
      if (event.data == "close") {
        close();
      }
    });
    iframe = document.createElement("iframe");
    iframe.onload = () => {
      const payload = { name: self.name, opener: !!self.opener, openerDOMAccess: openerDOMAccessAllowed };
      iframe.contentWindow.postMessage(payload, "*");
    };
    const channelName = new URL(location).searchParams.get("channel");
    iframe.src = `${get_host_info().HTTPS_ORIGIN}/html/cross-origin-opener-policy/resources/postback.html?channel=${channelName}`;
    document.body.appendChild(iframe);
  }
</script>
"""
