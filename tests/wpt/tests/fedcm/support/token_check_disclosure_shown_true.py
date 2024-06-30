import importlib
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.tokenCheck(request)
  if (request_error):
    return request_error

  nonce = request.POST.get(b"nonce") or b""
  if request.POST.get(b"disclosure_text_shown") != b"true":
    return (560, [], "disclosure_text_shown is not true")
  if request.POST.get(b"disclosure_shown_for") != b"name,email,picture":
    return (561, [], "disclosure_shown_for is not name,email,picture")
  fields = request.POST.get(b"fields") or b""
  if fields != nonce:
    return (562, [], "fields does not match nonce")

  response.headers.set(b"Content-Type", b"application/json")
  response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"Origin"))
  response.headers.set(b"Access-Control-Allow-Credentials", "true")

  return "{\"token\": \"token\"}"
