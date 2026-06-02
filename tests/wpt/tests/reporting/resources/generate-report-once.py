def main(request, response):
  # Handle CORS preflight requests
  if request.method == u'OPTIONS':
    # Always reject preflights for one subdomain
    if b"www2" in request.headers[b"Origin"]:
      return (400, [], u"CORS preflight rejected for www2")
    return [
        (b"Content-Type", b"text/plain"),
        (b"Access-Control-Allow-Origin", b"*"),
        (b"Access-Control-Allow-Methods", b"get"),
        (b"Access-Control-Allow-Headers", b"Content-Type"),
    ], u"CORS allowed"

  if b"reportID" in request.GET:
    key = request.GET.first(b"reportID")
  else:
    response.status = 400
    return "reportID parameter is required."

  with request.server.stash.lock:
    visited = request.server.stash.take(key=key)
    if visited is None:
      response.headers.set("Reporting-Endpoints",
                           b"default=\"/reporting/resources/report.py?reportID=%s\"" % key)
    request.server.stash.put(key=key, value=True)

  response.content = b"""
<!DOCTYPE HTML>
<meta charset=utf-8>
<title>Generate deprecation report</title>
<script>
  webkitRequestAnimationFrame(() => {});
</script>
"""
