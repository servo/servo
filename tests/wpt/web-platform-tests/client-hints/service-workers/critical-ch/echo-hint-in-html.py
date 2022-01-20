import sys

def main(request, response):
    """
    Simple handler that sets a response header based on which client hint
    request headers were received.
    """

    response.headers.append(b"Content-Type", b"text/html; charset=UTF-8")
    response.headers.append(b"Access-Control-Allow-Origin", b"*")
    response.headers.append(b"Access-Control-Allow-Headers", b"*")
    response.headers.append(b"Access-Control-Expose-Headers", b"*")

    response.headers.append(b"Cache-Control", b"no-store")

    response.headers.append(b"Accept-CH", b"sec-ch-device-memory,device-memory");
    response.headers.append(b"Critical-CH", b"sec-ch-device-memory,device-memory");

    result = "FAIL"

    if b"sec-ch-device-memory" in request.headers and b"device-memory" in request.headers:
      result = "PASS"

    response.content = result
