from cookies.resources.helpers import setNoCacheAndCORSHeaders
def main(request, response):
    # Step 4 (partitioned-popins/partitioned-popins.request-header.tentative.https.window.js)
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/html")
    headers.append((b'Popin-Policy', b"partitioned=*"))
    message = request.GET[b'message']
    message += b"HTTP("
    message += request.headers.get(b"Sec-Popin-Context", b"missing")
    message += b")-"
    document = b"""
<!doctype html>
<meta charset="utf-8">
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
<script>
(async function() {
  test_driver.set_test_context(window.opener);
  window.location = "/partitioned-popins/resources/partitioned-popins.request-header.js.py?message=""" + message + b"""";
})();
</script>
"""
    return headers, document