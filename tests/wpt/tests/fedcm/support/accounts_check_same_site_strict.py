import importlib
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.accountsCheck(request)
  if (request_error):
    return request_error
  if request.cookies.get(b"same_site_strict") == b"1":
    return (546, [], "Should not send SameSite=Strict cookies")
  # TODO(crbug.com/350944661): We want to send these cookies.
  if request.cookies.get(b"same_site_lax") == b"1":
    return (547, [], "Should not send SameSite=Lax cookies")
  if request.headers.get(b"Sec-Fetch-Site") != b"cross-site":
    return (538, [], "Wrong Sec-Fetch-Site header")

  response.headers.set(b"Content-Type", b"application/json")

  return """
{
 "accounts": [{
   "id": "1234",
   "given_name": "John",
   "name": "John Doe",
   "email": "john_doe@idp.example",
   "picture": "https://idp.example/profile/123",
   "approved_clients": ["123", "456", "789"],
   "login_hints": ["john_doe"],
   "domain_hints": ["idp.example", "example"]
  }]
}
"""
