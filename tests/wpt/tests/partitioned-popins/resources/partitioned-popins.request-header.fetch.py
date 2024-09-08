from cookies.resources.helpers import setNoCacheAndCORSHeaders
def main(request, response):
    # Step 7 (partitioned-popins/partitioned-popins.request-header.tentative.https.window.js)
    headers = setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"application/json")
    message = request.GET[b'message']
    message += b"fetch("
    message += request.headers.get(b"Sec-Popin-Context", b"missing")
    message += b")-"
    return headers, b'{"message": "' + message + b'"}'