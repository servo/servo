from cookies.resources.helpers import makeCookieHeader, readCookies, setNoCacheAndCORSHeaders

# Step 4/6 (workers/same-site-cookies/{})
def main(request, response):
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/javascript")
    headers.append(makeCookieHeader(b"samesite_strict_set_on_load", b"test", {b"SameSite":b"Strict", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(b"samesite_lax_set_on_load", b"test", {b"SameSite":b"Lax", b"path":b"/", b"Secure":b""}))
    headers.append(makeCookieHeader(b"samesite_none_set_on_load", b"test", {b"SameSite":b"None", b"path":b"/", b"Secure":b""}))
    cookies = readCookies(request)
    message = b"ReadOnLoad:"
    if b"samesite_strict_set_before_load" in cookies:
        message += b"Strict"
    if b"samesite_lax_set_before_load" in cookies:
        message += b"Lax"
    if b"samesite_none_set_before_load" in cookies:
        message += b"None"
    document = b"""
self.onconnect = (e) => {
    fetch("/workers/same-site-cookies/resources/get_cookies_redirect.py", {credentials: 'include'}).then((resp) => {
        resp.json().then((cookies) => {
            let message = \"""" + message + b""",ReadOnFetch:";
            if (cookies.hasOwnProperty("samesite_strict_set_before_load")) {
                message += "Strict";
            }
            if (cookies.hasOwnProperty("samesite_lax_set_before_load")) {
                message += "Lax";
            }
            if (cookies.hasOwnProperty("samesite_none_set_before_load")) {
                message += "None";
            }
            e.ports[0].postMessage(message);
            self.close();
        });
    });
}
"""
    return headers, document
