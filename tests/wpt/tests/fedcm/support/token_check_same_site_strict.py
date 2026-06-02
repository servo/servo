import importlib
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.tokenCheck(request)
  if (request_error):
    return request_error
  if request.cookies.get(b"same_site_strict") == b"1":
    return (546, [], "Should not send SameSite=Strict cookies")
  # TODO(crbug.com/350944661): We want to send these cookies.
  if request.cookies.get(b"same_site_lax") == b"1":
    return (547, [], "Should not send SameSite=Lax cookies")

  response.headers.set(b"Content-Type", b"application/json")
  response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"Origin"))
  response.headers.set(b"Access-Control-Allow-Credentials", "true")

  return "{\"token\": \"token\"}"
