import time
import json

def main(request, response):

    op = request.GET.first("op");
    key = request.GET.first("reportID")

    if op == "take":
      timeout = float(request.GET.first("timeout"))
      value = request.server.stash.take(key=key)
      if value is not None:
          return [("Content-Type", "application/json")], value
      else:
          time.sleep(timeout)
          value = request.server.stash.take(key=key)
          if value is not None:
            return [("Content-Type", "application/json")], value
          else:
            return [("Content-Type", "application/json")], json.dumps({'error': 'No such report.' , 'guid' : key})
    else:
        report = request.body
        report.rstrip()
        request.server.stash.take(key=key)
        request.server.stash.put(key=key, value=report)
        return [("Content-Type", "text/plain")], "Recorded report " + report