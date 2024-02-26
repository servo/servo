from cookies.resources.helpers import makeCookieHeader, readCookies, setNoCacheAndCORSHeaders

# This worker messages how many connections have been made and checks what cookies are available.
def main(request, response):
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/javascript")
    cookies = readCookies(request)
    message = b"ReadOnLoad:"
    if b"samesite_strict" in cookies:
        message += b"Strict"
    if b"samesite_lax" in cookies:
        message += b"Lax"
    if b"samesite_none" in cookies:
        message += b"None"
    document = b"""
let connection_count = 0;
self.onconnect = (e) => {
    connection_count++;
    fetch("/storage-access-api/resources/get_cookies.py", {credentials: 'include'}).then((resp) => {
        resp.json().then((cookies) => {
            let message = \"""" + message + b""",ReadOnFetch:";
            if (cookies.hasOwnProperty("samesite_strict")) {
                message += "Strict";
            }
            if (cookies.hasOwnProperty("samesite_lax")) {
                message += "Lax";
            }
            if (cookies.hasOwnProperty("samesite_none")) {
                message += "None";
            }
            message += ",ConnectionsMade:" + connection_count;
            e.ports[0].postMessage(message);
        });
    });
}
"""
    return headers, document