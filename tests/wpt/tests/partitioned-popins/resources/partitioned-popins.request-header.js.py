from cookies.resources.helpers import setNoCacheAndCORSHeaders
def main(request, response):
    # Step 5 (partitioned-popins/partitioned-popins.request-header.tentative.https.window.js)
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/html")
    headers.append((b'Popin-Policy', b"partitioned=*"))
    message = request.GET[b'message']
    message += b"JS("
    message += request.headers.get(b"Sec-Popin-Context", b"missing")
    message += b")-"
    document = b"""
<!doctype html>
<meta charset="utf-8">
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
<body>
<script>
(async function() {
  test_driver.set_test_context(window.opener);
  let message = \"""" + message + b"""\";

  // Step 6 (partitioned-popins/partitioned-popins.request-header.tentative.https.window.js)
  const response = await fetch("/partitioned-popins/resources/partitioned-popins.request-header.fetch.py?message=" + message);
  const json = await response.json();
  message = json["message"];

  // Step 8 (partitioned-popins/partitioned-popins.request-header.tentative.https.window.js)
  const iframe = document.createElement("iframe");
  iframe.src = "/partitioned-popins/resources/partitioned-popins.request-header.iframe.py?message=" + message;
  document.body.appendChild(iframe);
})();
</script>
</body>
"""
    return headers, document