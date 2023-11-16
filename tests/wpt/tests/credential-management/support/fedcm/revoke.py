import importlib
error_checker = importlib.import_module("credential-management.support.fedcm.request-params-check")

def main(request, response):
  request_error = error_checker.revokeCheck(request)
  if (request_error):
    return request_error

  response.headers.set(b"Content-Type", b"application/json")

  # Pass the account_hint as the accountId.
  account_hint = request.POST.get(b"account_hint");
  return f"{{\"account_id\": \"{account_hint}\"}}"
