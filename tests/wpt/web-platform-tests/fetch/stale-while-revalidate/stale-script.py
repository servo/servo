import random, string, datetime

def id_token():
   letters = string.ascii_lowercase
   return ''.join(random.choice(letters) for i in range(20))

def main(request, response):
    token = request.GET.first("token", None)
    is_query = request.GET.first("query", None) != None
    with request.server.stash.lock:
      value = request.server.stash.take(token)
      count = 0
      if value != None:
        count = int(value)
      if is_query:
        if count < 2:
          request.server.stash.put(token, count)
      else:
        count = count + 1
        request.server.stash.put(token, count)

    if is_query:
      headers = [("Count", count)]
      content = ""
      return 200, headers, content
    else:
      unique_id = id_token()
      headers = [("Content-Type", "text/javascript"),
                 ("Cache-Control", "private, max-age=0, stale-while-revalidate=60"),
                 ("Unique-Id", unique_id)]
      content = "report('{}')".format(unique_id)
      return 200, headers, content
