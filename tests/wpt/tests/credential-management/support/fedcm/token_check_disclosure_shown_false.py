import importlib
error_checker = importlib.import_module("credential-management.support.fedcm.request-params-check")

def main(request, response):
  request_error = error_checker.tokenCheck(request)
  if (request_error):
    return request_error

  if request.POST.get(b"disclosure_text_shown") != b"false":
    return (560, [], "disclosure_text_shown is not false")

  response.headers.set(b"Content-Type", b"application/json")
  response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"Origin"))
  response.headers.set(b"Access-Control-Allow-Credentials", "true")

  return "{\"token\": \"token\"}"
