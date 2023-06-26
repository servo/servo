def main(request, response):
    coop = request.GET.first(b"coop")
    coep = request.GET.first(b"coep")
    sandbox = request.GET.first(b"sandbox")
    if coop != "":
        response.headers.set(b"Cross-Origin-Opener-Policy", coop)
    if coep != "":
        response.headers.set(b"Cross-Origin-Embedder-Policy", coep)
    response.headers.set(b"Content-Security-Policy", b"sandbox " + sandbox + b";")

    # Open a popup to coop-coep.py with the same parameters (except sandbox)
    response.content = b"""
<!doctype html>
<meta charset=utf-8>
<script src="/common/get-host-info.sub.js"></script>
<script src="/html/cross-origin-opener-policy/resources/fully-loaded.js"></script>
<script>
  const params = new URL(location).searchParams;
  params.delete("sandbox");
  const navigate = params.get("navigate");
  if (navigate) {
    fullyLoaded().then(() => {
      self.location = navigate;
    });
  } else {
    window.open(`/html/cross-origin-opener-policy/resources/coop-coep.py?${params}`);
  }
</script>
"""
