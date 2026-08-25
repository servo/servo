import importlib

def main(request, response):
  response.headers.set(b"Content-Type", b"application/json")
  response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
  response.headers.set(b"Access-Control-Allow-Credentials", b"true")

  return (599, [], "Server failure")
