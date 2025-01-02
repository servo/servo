from cookies.resources.helpers import setNoCacheAndCORSHeaders
def main(request, response):
    # Step 9 (partitioned-popins/partitioned-popins.request-header.tentative.https.window.js)
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/html")
    message = request.GET[b'message']
    message += b"iframe("
    message += request.headers.get(b"Sec-Popin-Context", b"missing")
    message += b")-"
    document = b"""
<!doctype html>
<meta charset="utf-8">
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
<script>
(async function() {
  test_driver.set_test_context(window.top.opener);
  window.top.opener.postMessage({type: "popin", message: \"""" + message + b"""\"}, "*");
})();
</script>
"""
    return headers, document