import importlib
error_checker = importlib.import_module("credential-management.support.fedcm.request-params-check")

def main(request, response):
  request_error = error_checker.tokenCheck(request)
  if (request_error):
    return request_error

  response.headers.set(b"Content-Type", b"application/json")

  return "{\"token\": \"token\"}"
