from cookies.resources.helpers import makeCookieHeader

# Step 3/5 (workers/same-site-cookies/{})
def main(request, response):
    headers = [(b"Location", b"/workers/same-site-cookies/resources/get_cookies.py")]
    headers.append(makeCookieHeader(b"samesite_strict_set_on_redirect_fetch", b"test", {b"SameSite":b"Strict", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(b"samesite_lax_set_on_redirect_fetch", b"test", {b"SameSite":b"Lax", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(b"samesite_none_set_on_redirect_fetch", b"test", {b"SameSite":b"None", b"path":b"/", b"Secure":b""}))
    return 302, headers, b""
