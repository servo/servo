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

    accept = b"sec-ch-device-memory,device-memory"
    if(request.GET.first(b"multiple", None) is not None):
      for accept_part in accept.split(b","):
        response.headers.append(b"Accept-CH", accept_part)
    else:
      response.headers.append(b"Accept-CH", accept)

    critical = b"sec-ch-device-memory,device-memory"
    if(request.GET.first(b"mismatch", None) is not None):
      critical = b"sec-ch-viewport-width,viewport-width"

    if(request.GET.first(b"multiple", None) is not None):
      for critical_part in critical.split(b","):
        response.headers.append(b"Critical-CH", critical_part)
    else:
      response.headers.append(b"Critical-CH", critical)

    response.headers.append(b"Cache-Control", b"no-store")

    result = "FAIL"

    if b"sec-ch-device-memory" in request.headers and b"device-memory" in request.headers:
      result = "PASS"

    token = request.GET.first(b"token", None)
    if(token is not None):
      with request.server.stash.lock:
        count = request.server.stash.take(token)
        if(count == None):
          count = 1
        else:
          count += 1
        request.server.stash.put(token, count)
        result = str(count)

    if b"sec-ch-viewport-width" in request.headers and b"viewport-width" in request.headers:
      result = "MISMATCH"

    response.content = "<script>(window.opener || window.top).postMessage('{0}', '*')</script>".format(result)
