def main(request, response):
  command = request.GET.first("cmd").lower();
  test_id = request.GET.first("id")
  if command == "put":
    request.server.stash.put(test_id, request.headers.get("Content-Type", ""))
    return [("Content-Type", "text/plain")], ""

  if command == "get":
    stashed_header = request.server.stash.take(test_id)
    if stashed_header is not None:
      return [("Content-Type", "text/plain")], stashed_header

  response.set_error(400, "Bad Command")
  return "ERROR: Bad Command!"
