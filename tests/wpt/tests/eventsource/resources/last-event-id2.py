ID_PERSISTS = 1
ID_RESETS_1 = 2
ID_RESETS_2 = 3

def main(request, response):
  response.headers.set(b"Content-Type", b"text/event-stream")
  try:
    test_type = int(request.GET.first(b"type", ID_PERSISTS))
  except:
    test_type = ID_PERSISTS

  if test_type == ID_PERSISTS:
    return b"id: 1\ndata: 1\n\ndata: 2\n\nid: 2\ndata:3\n\ndata:4\n\n"

  elif test_type == ID_RESETS_1:
    return b"id: 1\ndata: 1\n\nid:\ndata:2\n\ndata:3\n\n"

  # empty id field without colon character (:) should also reset
  elif test_type == ID_RESETS_2:
    return b"id: 1\ndata: 1\n\nid\ndata:2\n\ndata:3\n\n"

  else:
    return b"data: invalid_test\n\n"
