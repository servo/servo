from cookies.resources.helpers import makeCookieHeader, readCookies, setNoCacheAndCORSHeaders
def main(request, response):
    id = request.GET[b'id']
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/html")
    cookies = readCookies(request)
    message = b"ReadOnLoad:"
    if cookies.get(b"first-party-strict") == id:
        message += b"FirstPartyStrict-"
    if cookies.get(b"first-party-lax") == id:
        message += b"FirstPartyLax-"
    if cookies.get(b"first-party-none") == id:
        message += b"FirstPartyNone-"
    if cookies.get(b"third-party-strict") == id:
        message += b"ThirdPartyStrict-"
    if cookies.get(b"third-party-lax") == id:
        message += b"ThirdPartyLax-"
    if cookies.get(b"third-party-none") == id:
        message += b"ThirdPartyNone-"
    document = b"""
<!doctype html>
<meta charset="utf-8">
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
<script>
(async function() {
  // Step 7 (partitioned-popins/partitioned-popins.cookies.tentative.sub.https.window.js)
  const id = (new URLSearchParams(window.location.search)).get("id");
  test_driver.set_test_context(window.top);
  let resp = await fetch("/partitioned-popins/resources/get_cookies.py", {credentials: 'include'});
  let resp_json = await resp.json();
  let message = \"""" + message + b""",ReadOnFetch:";
  if (resp_json["first-party-strict"] == id) {
    message += "FirstPartyStrict-";
  }
  if (resp_json["first-party-lax"] == id) {
    message += "FirstPartyLax-";
  }
  if (resp_json["first-party-none"] == id) {
    message += "FirstPartyNone-";
  }
  if (resp_json["third-party-strict"] == id) {
    message += "ThirdPartyStrict-";
  }
  if (resp_json["third-party-lax"] == id) {
    message += "ThirdPartyLax-";
  }
  if (resp_json["third-party-none"] == id) {
    message += "ThirdPartyNone-";
  }
  window.opener.postMessage({type: "popin-read", message: message}, "*");
  window.close();
})();
</script>
"""
    return headers, document