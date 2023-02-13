def main(request, response):
  if request.cookies.get(b"cookie") != b"1":
    return (530, [], "Missing cookie")
  if request.headers.get(b"Accept") != b"application/json":
    return (531, [], "Wrong Accept")
  if request.headers.get(b"Sec-Fetch-Dest") != b"webidentity":
    return (532, [], "Wrong Sec-Fetch-Dest header")
  if request.headers.get(b"Referer"):
    return (533, [], "Should not have Referer")
  if request.headers.get(b"Origin"):
    return (534, [], "Should not have Origin")

  return """
{
 "accounts": [
  {
   "id": "john_doe",
   "given_name": "John",
   "name": "John Doe",
   "email": "john_doe@idp.example",
   "picture": "https://idp.example/profile/123",
   "approved_clients": ["123", "456", "789"]
  }
  ]
}
"""
