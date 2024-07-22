import importlib
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.tokenCheck(request)
  if (request_error):
    return request_error

  response.headers.set(b"Content-Type", b"application/json")
  if b"nocors" not in request.GET:
    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"Origin"))
    response.headers.set(b"Access-Control-Allow-Credentials", "true")

  return "{\"token\": \"token\"}"
