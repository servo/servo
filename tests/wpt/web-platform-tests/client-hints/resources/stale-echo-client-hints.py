import random
import string

def id_token():
   letters = string.ascii_lowercase
   return ''.join(random.choice(letters) for i in range(20))

def main(request, response):
    client_hint_headers = [
      "device-memory",
      "dpr",
      "width",
      "viewport-width",
      "rtt",
      "downlink",
      "ect",
      "sec-ch-lang",
      "sec-ch-ua",
      "sec-ch-ua-arch",
      "sec-ch-ua-platform",
      "sec-ch-ua-model",
    ]

    client_hints_curr = {i:request.headers.get(i) for i in client_hint_headers}

    token = request.GET.first("token", None)
    is_query = request.GET.first("query", None) is not None
    with request.server.stash.lock:
      stash = request.server.stash.take(token)
      if stash != None:
        (value, client_hints_prev) = stash
        count = int(value)
      else:
        count = 0
        client_hints_prev = {}

      if is_query:
        if count < 2:
          request.server.stash.put(token, (count, client_hints_curr))
      else:
        count = count + 1
        request.server.stash.put(token, (count, client_hints_curr))

    for header in client_hint_headers:
      if client_hints_curr[header] is not None:
        response.headers.set(header+"-recieved", client_hints_curr[header])
      if (header in client_hints_prev) and (client_hints_prev[header] is not None):
        response.headers.set(header+"-previous", client_hints_prev[header])

    if is_query:
      headers = [("Count", count)]
      content = ""
      return 200, headers, content
    else:
      unique_id = id_token()
      headers = [("Content-Type", "text/html"),
                 ("Cache-Control", "private, max-age=0, stale-while-revalidate=60"),
                 ("Unique-Id", unique_id)]
      content = "report('{}')".format(unique_id)
      return 200, headers, content