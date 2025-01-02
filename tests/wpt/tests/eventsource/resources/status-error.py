def main(request, response):
  status = (request.GET.first(b"status", b"404"), b"HAHAHAHA")
  headers = [(b"Content-Type", b"text/event-stream")]

  # According to RFC7231, HTTP responses bearing status code 204 or 205 must
  # not specify a body. The expected browser behavior for this condition is not
  # currently defined--see the following for further discussion:
  #
  # https://github.com/web-platform-tests/wpt/pull/5227
  if status[0] in [b"204", b"205"]:
      body = b""
  else:
      body = b"data: data\n\n"

  return status, headers, body
