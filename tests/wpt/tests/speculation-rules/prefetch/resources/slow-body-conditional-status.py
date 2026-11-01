import os.path
import time

from wptserve.pipes import template

def main(request, response):
  prefetch = request.headers.get("Sec-Purpose", b"").decode("utf-8").startswith("prefetch")

  response.headers.set(b"Content-Type", b"text/html")
  response.headers.set(b"Cache-Control", b"no-store")

  executor = template(
    request,
    open(os.path.join(os.path.dirname(__file__), "executor.sub.html"), "rb").read())

  if prefetch:
    # Return 503 status with headers immediately, but delay the body.
    # This creates a window where the response status (503) is set on the
    # cached resource, but notifyFinished hasn't run yet (still loading body),
    # so the memory cache cleanup hasn't happened.
    response.status = 503
    response.write_status_headers()
    time.sleep(2)
    response.writer.write_content(executor + b"<body>503")
  else:
    response.status = 200
    response.content = executor + b"<body>200"
