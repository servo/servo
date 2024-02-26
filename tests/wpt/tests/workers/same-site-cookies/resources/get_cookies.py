import json
from cookies.resources.helpers import makeCookieHeader, readCookies, setNoCacheAndCORSHeaders

from wptserve.utils import isomorphic_decode

def main(request, response):
    headers = setNoCacheAndCORSHeaders(request, response)
    cookies = readCookies(request)
    headers.append(makeCookieHeader(b"samesite_strict_set_on_fetch", b"test", {b"SameSite":b"Strict", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(b"samesite_lax_set_on_fetch", b"test", {b"SameSite":b"Lax", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(b"samesite_none_set_on_fetch", b"test", {b"SameSite":b"None", b"path":b"/", b"Secure":b""}))
    decoded_cookies = {isomorphic_decode(key): isomorphic_decode(val) for key, val in cookies.items()}
    return headers, json.dumps(decoded_cookies)
