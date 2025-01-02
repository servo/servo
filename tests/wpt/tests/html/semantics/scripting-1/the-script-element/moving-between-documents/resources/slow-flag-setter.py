
import time

def main(request, response):
  headers = [(b"Content-Type", b"text/javascript")]

  result = request.GET.first(b"result", b"success")
  if result == b"css":
    time.sleep(3)
    headers = [(b"Content-Type", b"text/css")]
    body = u""
  else:
    time.sleep(2)

    body = u"""
      fetch('exec');
      console.log('exec');
      if (!window.readyToEvaluate) {
        window.didExecute = "executed too early";
      } else {
        window.didExecute = "executed";
      }
    """
    if result == b"parse-error":
      body = u"1=2 parse error;"
    if result == b"fetch-error":
      return 404, [(b'Content-Type', b'text/plain')], u"""window.didExecute = "fetch error";"""

  return headers, body
