import time

def main(request, response):
  time.sleep(0.1)
  return [("Content-Type", "text/plain")], "FAIL"
