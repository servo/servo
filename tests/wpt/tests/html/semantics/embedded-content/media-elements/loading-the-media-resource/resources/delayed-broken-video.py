import time

def main(request, response):
  time.sleep(0.1)
  return [(b"Content-Type", b"text/plain")], u"FAIL"
