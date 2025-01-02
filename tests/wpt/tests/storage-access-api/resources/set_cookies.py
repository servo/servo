from cookies.resources.helpers import makeCookieHeader, setNoCacheAndCORSHeaders

def main(request, response):
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/html; charset=utf-8")
    headers.append(makeCookieHeader(b"samesite_strict", b"test", {b"SameSite":b"Strict", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(b"samesite_lax", b"test", {b"SameSite":b"Lax", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(b"samesite_none", b"test", {b"SameSite":b"None", b"path":b"/", b"Secure":b""}))
    document = b"<!DOCTYPE html>"
    return headers, document
