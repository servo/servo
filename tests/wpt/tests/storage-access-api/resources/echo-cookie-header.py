def main(request, response):
  # Set the cors enabled headers.
  origin = request.headers.get(b"Origin")
  if origin:
      response.headers.set(b"Content-Type", b"text/plain")
      response.headers.set(b"Access-Control-Allow-Origin", origin)
      response.headers.set(b"Access-Control-Allow-Credentials", 'true')

  return request.headers.get(b"Cookie", b"")
