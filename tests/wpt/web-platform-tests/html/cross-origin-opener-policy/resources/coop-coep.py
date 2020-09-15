def main(request, response):
    coop = request.GET.first(b"coop")
    coopReportOnly = request.GET.first(b"coop-report-only", None)
    coep = request.GET.first(b"coep")
    coepReportOnly = request.GET.first(b"coep-report-only", None)
    redirect = request.GET.first(b"redirect", None)
    if coop != b"":
        response.headers.set(b"Cross-Origin-Opener-Policy", coop)
    if coopReportOnly is not None:
        response.headers.set(b"Cross-Origin-Opener-Policy-Report-Only", coopReportOnly)
    if coep != b"":
        response.headers.set(b"Cross-Origin-Embedder-Policy", coep)
    if coepReportOnly is not None:
        response.headers.set(b"Cross-Origin-Embedder-Policy-Report-Only", coepReportOnly)
    if b'cache' in request.GET:
        response.headers.set(b'Cache-Control', b'max-age=3600')
    host = request.url_parts[1]

    if redirect != None:
        response.status = 302
        response.headers.set(b"Location", redirect)
        return

    # This uses an <iframe> as BroadcastChannel is same-origin bound.
    response.content = b"""
<!doctype html>
<meta charset=utf-8>
<script src="/common/get-host-info.sub.js"></script>
<script src="/html/cross-origin-opener-policy/resources/common.js"></script>
<body>
<script>
  const params = new URL(location).searchParams;
  const navHistory = params.get("navHistory");
  const avoidBackAndForth = params.get("avoidBackAndForth");
  const navigate = params.get("navigate");
  if (navHistory !== null) {
    fullyLoaded().then(() => {
      history.go(Number(navHistory));
    });
  } else if (navigate !== null && (history.length === 1 || !avoidBackAndForth)) {
    fullyLoaded().then(() => {
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
    iframe.src = `${get_host_info().HTTPS_ORIGIN}/html/cross-origin-opener-policy/resources/postback.html?channel=${encodeURIComponent(channelName)}`;
    document.body.appendChild(iframe);
  }
</script>
</body>
"""
