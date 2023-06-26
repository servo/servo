def main(request, response):
  if len(request.cookies) > 0:
    return (530, [], "Cookie should not be sent to manifest endpoint")
  if request.headers.get(b"Accept") != b"application/json":
    return (531, [], "Wrong Accept")
  if request.headers.get(b"Sec-Fetch-Dest") != b"webidentity":
    return (532, [], "Wrong Sec-Fetch-Dest header")
  if request.headers.get(b"Referer"):
    return (533, [], "Should not have Referer")
  if request.headers.get(b"Origin"):
    return (534, [], "Should not have Origin")

  response.headers.set(b"Content-Type", b"application/json")

  return """
{
  "accounts_endpoint": "accounts.py",
  "client_metadata_endpoint": "client_metadata.py",
  "id_assertion_endpoint": "token.py",
  "revocation_endpoint": "revoke.py"
}
"""
