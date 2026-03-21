import importlib
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.tokenCheck(request)
  if (request_error):
    return request_error

  response.headers.set(b"Content-Type", b"application/json")
  response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"Origin"))
  response.headers.set(b"Access-Control-Allow-Credentials", "true")

  account = request.POST.get(b"account_id").decode("utf-8")
  nonce = request.POST.get(b"nonce")
  if nonce:
    nonce = nonce.decode("utf-8")
  else:
    nonce = ""
  params = request.POST.get(b"params")
  if params:
    params = params.decode("utf-8")
  else:
    params = ""
  if nonce == "token":
    return "{\"token\": \"account=%s\"}" % (account)
  return "{\"continue_on\": \"resolve.html?selected=%s&%s&%s\"}" % (account, nonce, params)
