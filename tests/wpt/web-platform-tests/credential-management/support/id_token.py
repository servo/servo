def main(request, response):
  if not b"cookie" in request.cookies or request.cookies[b"cookie"].value != b"1":
    return (500, [], "Missing cookie")
  return "{\"id_token\": \"token\"}"
