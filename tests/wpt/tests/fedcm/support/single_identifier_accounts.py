import importlib
error_checker = importlib.import_module("fedcm.support.request-params-check")

def main(request, response):
  request_error = error_checker.accountsCheck(request)
  if (request_error):
    return request_error

  response.headers.set(b"Content-Type", b"application/json")

  return """
{
 "accounts": [
  {
   "id": "john_doe",
   "username": "john_doe"
  },
  {
   "id": "john_doe2",
   "tel": "+1 (416) 000-0000"
  }
  ]
}
"""
