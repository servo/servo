import random
import string

from wptserve.utils import isomorphic_encode
import importlib
client_hints_full_list = importlib.import_module("client-hints.resources.clienthintslist").client_hints_full_list

def id_token():
   letters = string.ascii_lowercase
   return u''.join(random.choice(letters) for i in range(20))

def main(request, response):
    client_hint_headers = client_hints_full_list()
    client_hints_curr = {i:request.headers.get(i) for i in client_hint_headers}

    token = request.GET.first(b"token", None)
    is_query = request.GET.first(b"query", None) is not None
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
        response.headers.set(header+b"-recieved", client_hints_curr[header])
      if (header in client_hints_prev) and (client_hints_prev[header] is not None):
        response.headers.set(header+b"-previous", client_hints_prev[header])

    if is_query:
      headers = [(b"Count", count)]
      content = u""
      return 200, headers, content
    else:
      unique_id = id_token()
      headers = [(b"Content-Type", b"text/html"),
                 (b"Cache-Control", b"private, max-age=0, stale-while-revalidate=60"),
                 (b"Unique-Id", isomorphic_encode(unique_id))]
      content = u"report('{}')".format(unique_id)
      return 200, headers, content
