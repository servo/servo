import random, string, datetime

def token():
   letters = string.ascii_lowercase
   return ''.join(random.choice(letters) for i in range(20))

def main(request, response):
    cookie = request.cookies.first("Count", None)
    count = 0
    if cookie != None:
      count = int(cookie.value)
    if request.GET.first("query", None) != None:
      headers = [("Count", count)]
      content = ""
      return 200, headers, content
    else:
      count = count + 1

      unique_id = token()
      headers = [("Content-Type", "text/javascript"),
                 ("Cache-Control", "private, max-age=0, stale-while-revalidate=10"),
                 ("Set-Cookie", "Count={}".format(count)),
                 ("Token", unique_id)]
      content = "report('{}')".format(unique_id)
      return 200, headers, content
