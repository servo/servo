import importlib
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.tokenCheck(request)
  if (request_error):
    return request_error

  response.headers.set(b"Content-Type", b"application/json")
  response.status = (401, b"Unauthorized")

  return "{\"error\": {\"code\": \"unauthorized_client\", \"url\": \"error.html\"}}"
