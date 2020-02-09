from helpers import makeCookieHeader, setNoCacheAndCORSHeaders

def main(request, response):
    """Respond to `/cookie/set/samesite?{value}` by setting four cookies:
    1. `samesite_strict={value};SameSite=Strict;path=/`
    2. `samesite_lax={value};SameSite=Lax;path=/`
    3. `samesite_none={value};SameSite=None;path=/`
    4. `samesite_unspecified={value};path=/`
    Then navigate to a page that will post a message back to the opener with the set cookies"""
    headers = setNoCacheAndCORSHeaders(request, response)
    value = request.url_parts.query

    headers.append(("Content-Type", "text/html; charset=utf-8"))
    headers.append(makeCookieHeader("samesite_strict", value, {"SameSite":"Strict","path":"/"}))
    headers.append(makeCookieHeader("samesite_lax", value, {"SameSite":"Lax","path":"/"}))
    # SameSite=None cookies must be Secure.
    headers.append(makeCookieHeader("samesite_none", value, {"SameSite":"None", "path":"/", "Secure": ""}))
    headers.append(makeCookieHeader("samesite_unspecified", value, {"path":"/"}))

    document = """
<!DOCTYPE html>
<script>
  // A same-site navigation, which should attach all cookies including SameSite ones.
  // This is necessary because this page may have been reached via a cross-site navigation, so
  // we might not have access to some SameSite cookies from here.
  window.location = "../samesite/resources/echo-cookies.html";
</script>
"""

    return headers, document
