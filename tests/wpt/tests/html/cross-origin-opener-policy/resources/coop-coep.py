import json

def main(request, response):
    requestData = request.GET
    if request.method == u"POST":
        requestData = request.POST

    coop = requestData.first(b"coop")
    coopReportOnly = requestData.first(b"coop-report-only", None)
    coep = requestData.first(b"coep")
    coepReportOnly = requestData.first(b"coep-report-only", None)
    redirect = requestData.first(b"redirect", None)
    if coop != b"":
        response.headers.set(b"Cross-Origin-Opener-Policy", coop)
    if coopReportOnly is not None:
        response.headers.set(b"Cross-Origin-Opener-Policy-Report-Only", coopReportOnly)
    if coep != b"":
        response.headers.set(b"Cross-Origin-Embedder-Policy", coep)
    if coepReportOnly is not None:
        response.headers.set(b"Cross-Origin-Embedder-Policy-Report-Only", coepReportOnly)
    if b'cache' in requestData:
        response.headers.set(b'Cache-Control', b'max-age=3600')
    host = request.url_parts[1]

    if redirect != None:
        response.status = 302
        response.headers.set(b"Location", redirect)
        return

    # Collect relevant params to be visible to response JS
    params = {}
    for key in (b"navHistory", b"avoidBackAndForth", b"navigate", b"channel", b"responseToken", b"iframeToken"):
        value = requestData.first(key, None)
        params[key.decode()] = value and value.decode()

    response.content = b"""
<!doctype html>
<meta charset=utf-8>
<script src="/common/get-host-info.sub.js"></script>
<script src="/html/cross-origin-opener-policy/resources/fully-loaded.js"></script>
<body>
<script>
  const params = %s;
  const navHistory = params.navHistory;
  const avoidBackAndForth = params.avoidBackAndForth;
  const navigate = params.navigate;
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
    const channelName = params.channel;
    const responseToken = params.responseToken;
    const iframeToken = params.iframeToken;
    iframe.src = `${get_host_info().HTTPS_ORIGIN}/html/cross-origin-opener-policy/resources/postback.html` +
                 `?channel=${encodeURIComponent(channelName)}` +
                 `&responseToken=${responseToken}` +
                 `&iframeToken=${iframeToken}`;
    document.body.appendChild(iframe);
  }
</script>
</body>
""" % json.dumps(params).encode("utf-8")
