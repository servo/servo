from cookies.resources.helpers import makeCookieHeader, readCookies, setNoCacheAndCORSHeaders
def main(request, response):
    id = request.GET[b'id']
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/html")
    headers.append((b'Popin-Policy', b"partitioned=*"))
    cookies = readCookies(request)
    decoded_cookies = [key + b"=" + val for key, val in cookies.items()]
    cookie_string = b";".join(decoded_cookies)
    document = b"""
<!doctype html>
<meta charset="utf-8">
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
<script src="/partitioned-popins/resources/cookie-helpers.js"></script>
<script>
(async function() {
  test_driver.set_test_context(window.opener);

  // Step 7 (partitioned-popins/partitioned-popins.cookies-*.tentative.sub.https.window.js)
  const id = (new URLSearchParams(window.location.search)).get("id");
  test_driver.set_test_context(window.top);
  let cookie_string_on_load = \"""" + cookie_string + b"""\";
  let message = "ReadOnLoad:";
  message += getCookieMessage(cookie_string_on_load, "FirstParty", "", id);
  message += getCookieMessage(cookie_string_on_load, "ThirdParty", "", id);
  document.cookie = "FirstPartyStrictPopin=" + id + "; SameSite=Strict; Secure";
  document.cookie = "FirstPartyLaxPopin=" + id + "; SameSite=Lax; Secure";
  document.cookie = "FirstPartyNonePopin=" + id + "; SameSite=None; Secure";
  document.cookie = "ThirdPartyStrictPopin=" + id + "; Partitioned; SameSite=Strict; Secure";
  document.cookie = "ThirdPartyLaxPopin=" + id + "; Partitioned; SameSite=Lax; Secure";
  document.cookie = "ThirdPartyNonePopin=" + id + "; Partitioned; SameSite=None; Secure";
  let resp = await fetch("/partitioned-popins/resources/get_cookies.py", {credentials: 'include'});
  let resp_text = await resp.text();
  message += ",ReadOnFetch:";
  message += getCookieMessage(resp_text, "FirstParty", "", id);
  message += getCookieMessage(resp_text, "ThirdParty", "", id);
  message += getCookieMessage(resp_text, "FirstParty", "Popin", id);
  message += getCookieMessage(resp_text, "ThirdParty", "Popin", id);
  message += ",ReadOnDocument:";
  message += getCookieMessage(document.cookie, "FirstParty", "", id);
  message += getCookieMessage(document.cookie, "ThirdParty", "", id);
  message += getCookieMessage(document.cookie, "FirstParty", "Popin", id);
  message += getCookieMessage(document.cookie, "ThirdParty", "Popin", id);
  await test_driver.set_permission({ name: 'storage-access' }, 'granted');
  await document.requestStorageAccess();
  document.cookie = "FirstPartyStrictPopinAfterRSA=" + id + "; SameSite=Strict; Secure";
  document.cookie = "FirstPartyLaxPopinAfterRSA=" + id + "; SameSite=Lax; Secure";
  document.cookie = "FirstPartyNonePopinAfterRSA=" + id + "; SameSite=None; Secure";
  document.cookie = "ThirdPartyStrictPopinAfterRSA=" + id + "; Partitioned; SameSite=Strict; Secure";
  document.cookie = "ThirdPartyLaxPopinAfterRSA=" + id + "; Partitioned; SameSite=Lax; Secure";
  document.cookie = "ThirdPartyNonePopinAfterRSA=" + id + "; Partitioned; SameSite=None; Secure";
  resp = await fetch("/partitioned-popins/resources/get_cookies.py", {credentials: 'include'});
  resp_text = await resp.text();
  message += ",ReadOnFetchAfterRSA:";
  message += getCookieMessage(resp_text, "FirstParty", "", id);
  message += getCookieMessage(resp_text, "ThirdParty", "", id);
  message += getCookieMessage(resp_text, "FirstParty", "Popin", id);
  message += getCookieMessage(resp_text, "ThirdParty", "Popin", id);
  message += getCookieMessage(resp_text, "FirstParty", "PopinAfterRSA", id);
  message += getCookieMessage(resp_text, "ThirdParty", "PopinAfterRSA", id);
  message += ",ReadOnDocumentAfterRSA:";
  message += getCookieMessage(document.cookie, "FirstParty", "", id);
  message += getCookieMessage(document.cookie, "ThirdParty", "", id);
  message += getCookieMessage(document.cookie, "FirstParty", "Popin", id);
  message += getCookieMessage(document.cookie, "ThirdParty", "Popin", id);
  message += getCookieMessage(document.cookie, "FirstParty", "PopinAfterRSA", id);
  message += getCookieMessage(document.cookie, "ThirdParty", "PopinAfterRSA", id);

  // Step 8 (partitioned-popins/partitioned-popins.cookies-*.tentative.sub.https.window.js)
  window.addEventListener("message", e => {
    switch (e.data.type) {
      case 'popin-iframe-read':
        message += e.data.message;
        window.opener.postMessage({type: "popin-read", message: message}, "*");
        window.close();
        break;
    }
  });
  const iframe = document.createElement("iframe");
  iframe.src = "https://{{hosts[][]}}:{{ports[https][0]}}/partitioned-popins/resources/partitioned-popins.cookies-popin-iframe.html?id="+id;
  document.body.appendChild(iframe);
})();
</script>
"""
    return headers, document