from cookies.resources.helpers import makeCookieHeader, readCookies, setNoCacheAndCORSHeaders
def main(request, response):
    id = request.GET[b'id']
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/html")
    headers.append((b'Popin-Policy', b"partitioned=*"))
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
  test_driver.set_test_context(window.opener);

  // Step 7 (partitioned-popins/partitioned-popins.cookies.tentative.sub.https.window.js)
  const id = (new URLSearchParams(window.location.search)).get("id");
  test_driver.set_test_context(window.top);
  document.cookie = "first-party-strict-popin=" + id + "; SameSite=Strict; Secure";
  document.cookie = "first-party-lax-popin=" + id + "; SameSite=Lax; Secure";
  document.cookie = "first-party-none-popin=" + id + "; SameSite=None; Secure";
  document.cookie = "third-party-strict-popin=" + id + "; Partitioned; SameSite=Strict; Secure";
  document.cookie = "third-party-lax-popin=" + id + "; Partitioned; SameSite=Lax; Secure";
  document.cookie = "third-party-none-popin=" + id + "; Partitioned; SameSite=None; Secure";
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
  if (resp_json["first-party-strict-popin"] == id) {
    message += "FirstPartyStrictPopin-";
  }
  if (resp_json["first-party-lax-popin"] == id) {
    message += "FirstPartyLaxPopin-";
  }
  if (resp_json["first-party-none-popin"] == id) {
    message += "FirstPartyNonePopin-";
  }
  if (resp_json["third-party-strict-popin"] == id) {
    message += "ThirdPartyStrictPopin-";
  }
  if (resp_json["third-party-lax-popin"] == id) {
    message += "ThirdPartyLaxPopin-";
  }
  if (resp_json["third-party-none-popin"] == id) {
    message += "ThirdPartyNonePopin-";
  }
  message += ",ReadOnDocument:";
  if (document.cookie.includes("first-party-strict="+id)) {
    message += "FirstPartyStrict-";
  }
  if (document.cookie.includes("first-party-lax="+id)) {
    message += "FirstPartyLax-";
  }
  if (document.cookie.includes("first-party-none="+id)) {
    message += "FirstPartyNone-";
  }
  if (document.cookie.includes("third-party-strict="+id)) {
    message += "ThirdPartyStrict-";
  }
  if (document.cookie.includes("third-party-lax="+id)) {
    message += "ThirdPartyLax-";
  }
  if (document.cookie.includes("third-party-none="+id)) {
    message += "ThirdPartyNone-";
  }
  if (document.cookie.includes("first-party-strict-popin="+id)) {
    message += "FirstPartyStrictPopin-";
  }
  if (document.cookie.includes("first-party-lax-popin="+id)) {
    message += "FirstPartyLaxPopin-";
  }
  if (document.cookie.includes("first-party-none-popin="+id)) {
    message += "FirstPartyNonePopin-";
  }
  if (document.cookie.includes("third-party-strict-popin="+id)) {
    message += "ThirdPartyStrictPopin-";
  }
  if (document.cookie.includes("third-party-lax-popin="+id)) {
    message += "ThirdPartyLaxPopin-";
  }
  if (document.cookie.includes("third-party-none-popin="+id)) {
    message += "ThirdPartyNonePopin-";
  }
  window.opener.postMessage({type: "popin-read", message: message}, "*");
  window.close();
})();
</script>
"""
    return headers, document