from cookies.resources.helpers import makeCookieHeader, setNoCacheAndCORSHeaders

from wptserve.utils import isomorphic_encode

def main(request, response):
    """Respond to `/cookie/set/samesite?{value}` by setting the following combination of cookies:
    1. `samesite_unsupported={value};SameSite=Unsupported;path=/;Secure`
    2. `samesite_unsupported_none={value};SameSite=Unsupported;SameSite=None;path=/;Secure`
    3. `samesite_unsupported_lax={value};SameSite=Unsupported;SameSite=Lax;path=/`
    4. `samesite_unsupported_strict={value};SameSite=Unsupported;SameSite=Strict;path=/`
    5. `samesite_none_unsupported={value};SameSite=None;SameSite=Unsupported;path=/;Secure`
    6. `samesite_lax_unsupported={value};SameSite=Lax;SameSite=Unsupported;path=/;Secure`
    7. `samesite_strict_unsupported={value};SameSite=Strict;SameSite=Unsupported;path=/;Secure`
    8. `samesite_lax_none={value};SameSite=Lax;SameSite=None;path=/;Secure`
    9. `samesite_lax_strict={value};SameSite=Lax;SameSite=Strict;path=/`
    10. `samesite_strict_lax={value};SameSite=Strict;SameSite=Lax;path=/`
    Then navigate to a page that will post a message back to the opener with the set cookies"""
    headers = setNoCacheAndCORSHeaders(request, response)
    value = isomorphic_encode(request.url_parts.query)

    headers.append((b"Content-Type", b"text/html; charset=utf-8"))
    # Unknown value; single attribute
    headers.append(makeCookieHeader(
        b"samesite_unsupported", value, {b"SameSite":b"Unsupported", b"path":b"/", b"Secure":b""}))

    # Multiple attributes; first attribute unknown
    headers.append(makeCookieHeader(
        b"samesite_unsupported_none", value, {b"SameSite":b"Unsupported", b"SameSite":b"None", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(
        b"samesite_unsupported_lax", value, {b"SameSite":b"Unsupported", b"SameSite":b"Lax", b"path":b"/"}))
    headers.append(makeCookieHeader(
        b"samesite_unsupported_strict", value, {b"SameSite":b"Unsupported", b"SameSite":b"Strict", b"path":b"/"}))

    # Multiple attributes; second attribute unknown
    headers.append(makeCookieHeader(
        b"samesite_none_unsupported", value, {b"SameSite":b"None", b"SameSite":b"Unsupported", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(
        b"samesite_lax_unsupported", value, {b"SameSite":b"Lax", b"SameSite":b"Unsupported", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(
        b"samesite_strict_unsupported", value, {b"SameSite":b"Strict", b"SameSite":b"Unsupported", b"path":b"/", b"Secure":b""}))

    # Multiple attributes; both known
    headers.append(makeCookieHeader(
        b"samesite_lax_none", value, {b"SameSite":b"Lax", b"SameSite":b"None", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(
        b"samesite_lax_strict", value, {b"SameSite":b"Lax", b"SameSite":b"Strict", b"path":b"/"}))
    headers.append(makeCookieHeader(
        b"samesite_strict_lax", value, {b"SameSite":b"Strict", b"SameSite":b"Lax", b"path":b"/"}))

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
