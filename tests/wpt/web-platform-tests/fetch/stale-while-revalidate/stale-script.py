import random, string, datetime

def id_token():
   letters = string.ascii_lowercase
   return ''.join(random.choice(letters) for i in range(20))

def main(request, response):
    token = request.GET.first("token", None)
    value = request.server.stash.take(token)
    count = 0
    if value != None:
      count = int(value)
    if request.GET.first("query", None) != None:
      headers = [("Count", count)]
      content = ""
      if count < 2:
        request.server.stash.put(token, count)
      return 200, headers, content
    else:
      count = count + 1

      unique_id = id_token()
      headers = [("Content-Type", "text/javascript"),
                 ("Cache-Control", "private, max-age=0, stale-while-revalidate=60"),
                 ("Unique-Id", unique_id)]
      content = "report('{}')".format(unique_id)
      request.server.stash.put(token, count)
      return 200, headers, content
