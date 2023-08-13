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
  if request.headers.get(b"Sec-Fetch-Mode") != b"no-cors":
    return (535, [], "Wrong Sec-Fetch-Mode header")
  if request.headers.get(b"Sec-Fetch-Site") != b"none":
    return (536, [], "Wrong Sec-Fetch-Site header")

  response.headers.set(b"Content-Type", b"application/json")

  return """
{
 "accounts": [
  ]
}
"""

