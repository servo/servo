
import time

def main(request, response):
  headers = [("Content-Type", "text/javascript")]

  result = request.GET.first("result", "success")
  if result == "css":
    time.sleep(3)
    headers = [("Content-Type", "text/css")]
    body = ""
  else:
    time.sleep(2)

    body = """
      fetch('exec');
      console.log('exec');
      if (!window.readyToEvaluate) {
        window.didExecute = "executed too early";
      } else {
        window.didExecute = "executed";
      }
    """
    if result == "parse-error":
      body = "1=2 parse error;"
    if result == "fetch-error":
      return 404, [('Content-Type', 'text/plain')], """window.didExecute = "fetch error";"""

  return headers, body
