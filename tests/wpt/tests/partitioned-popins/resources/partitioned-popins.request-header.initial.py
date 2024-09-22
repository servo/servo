from cookies.resources.helpers import setNoCacheAndCORSHeaders
def main(request, response):
    # Step 3 (partitioned-popins/partitioned-popins.request-header.tentative.https.window.js)
    message = b"Initial("
    message += request.headers.get(b"Sec-Popin-Context", b"missing")
    message += b")-"
    headers = setNoCacheAndCORSHeaders(request, response)
    headers.append((b'Location', b"/partitioned-popins/resources/partitioned-popins.request-header.http.py?message=" + message))
    headers.append((b'Popin-Policy', b"partitioned=*"))
    return 302, headers, b'{"redirect": true}'