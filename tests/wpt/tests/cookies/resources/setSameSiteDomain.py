from cookies.resources.helpers import makeCookieHeader, setNoCacheAndCORSHeaders

from wptserve.utils import isomorphic_encode

def main(request, response):
    """Respond to `/cookie/set/samesite?{value}` by setting four cookies:
    1. `samesite_strict={value};SameSite=Strict;path=/;domain={host}`
    2. `samesite_lax={value};SameSite=Lax;path=/;domain={host}`
    3. `samesite_none={value};SameSite=None;path=/;Secure;domain={host}`
    4. `samesite_unspecified={value};path=/;domain={host}`
    Where {host} is the hostname from which this page is served. (Requesting this resource
    without a Host header will result in a 500 server error.)
    Then navigate to a page that will post a message back to the opener with the set cookies"""
    headers = setNoCacheAndCORSHeaders(request, response)
    value = isomorphic_encode(request.url_parts.query)
    host_header = request.headers['host']
    hostname = host_header.split(b":")[0]
    host = isomorphic_encode(hostname)
    headers.append((b"Content-Type", b"text/html; charset=utf-8"))
    headers.append(makeCookieHeader(b"samesite_strict", value, {b"SameSite":b"Strict", b"path":b"/", b"domain":host}))
    headers.append(makeCookieHeader(b"samesite_lax", value, {b"SameSite":b"Lax", b"path":b"/", b"domain":host}))
    # SameSite=None cookies must be Secure.
    headers.append(makeCookieHeader(b"samesite_none", value, {b"SameSite":b"None", b"path":b"/", b"Secure": b"", b"domain":host}))
    headers.append(makeCookieHeader(b"samesite_unspecified", value, {b"path":b"/", b"domain":host}))

    document = b"""
<!DOCTYPE html>
<script>
  // A same-site navigation, which should attach all cookies including SameSite ones.
  // This is necessary because this page may have been reached via a cross-site navigation, so
  // we might not have access to some SameSite cookies from here.
  window.location = "../samesite/resources/echo-cookies.html";
</script>
"""

    return headers, document
