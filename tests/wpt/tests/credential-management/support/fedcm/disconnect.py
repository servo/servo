import importlib
error_checker = importlib.import_module("credential-management.support.fedcm.request-params-check")

def main(request, response):
  response.headers.set(b"Content-Type", b"application/json")
  response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
  response.headers.set(b"Access-Control-Allow-Credentials", b"true")
  request_error = error_checker.revokeCheck(request)
  if request_error:
    return request_error

  # Pass the account_hint as the accountId.
  account_hint = request.POST.get(b"account_hint")
  return f"{{\"account_id\": \"{account_hint}\"}}"
