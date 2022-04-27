def main(request, response):
  if not b"cookie" in request.cookies or request.cookies[b"cookie"].value != b"1":
    return (500, [], "Missing cookie")
  return """
{
 "accounts": [{
   "id": "1234",
   "given_name": "John",
   "name": "John Doe",
   "email": "john_doe@idp.example",
   "picture": "https://idp.example/profile/123",
   "approved_clients": ["123", "456", "789"]
  }]
}
"""
