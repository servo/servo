def main(request, response):
  if request.cookies.get(b"cookie") != b"1":
    return (530, [], "Missing cookie")
  if request.method != "POST":
    return (531, [], "Method is not POST")
  if request.headers.get(b"Content-Type") != b"application/x-www-form-urlencoded":
    return (532, [], "Wrong Content-Type")
  if request.headers.get(b"Accept") != b"application/json":
    return (533, [], "Wrong Accept")
  if request.headers.get(b"Sec-Fetch-Dest") != b"webidentity":
    return (500, [], "Wrong Sec-Fetch-Dest header")
  if not request.headers.get(b"Referer"):
    return (534, [], "Missing Referer")

  if not request.POST.get(b"client_id"):
    return (535, [], "Missing 'client_id' POST parameter")
  if not request.POST.get(b"account_id"):
    return (536, [], "Missing 'account_id' POST parameter")
  if not request.POST.get(b"disclosure_text_shown"):
    return (537, [], "Missing 'disclosure_text_shown' POST parameter")

  return "{\"token\": \"token\"}"
