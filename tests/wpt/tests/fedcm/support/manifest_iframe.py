import importlib
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.manifestCheck(request)
  if (request_error):
    return request_error

  response.headers.set(b"Content-Type", b"application/json")

  return """
{
  "accounts_endpoint": "accounts.py",
  "client_metadata_endpoint": "client_metadata_iframe.py",
  "id_assertion_endpoint": "token.py",
  "login_url": "login.html"
}
"""
