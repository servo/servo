def commonCheck(request, mode=b"no-cors"):
  if request.headers.get(b"Accept") != b"application/json":
    return (531, [], "Wrong Accept")
  if request.headers.get(b"Sec-Fetch-Dest") != b"webidentity":
    return (532, [], "Wrong Sec-Fetch-Dest header")
  if request.headers.get(b"Referer"):
    return (533, [], "Should not have Referer")
  if request.headers.get(b"Sec-Fetch-Mode") != mode:
    return (534, [], "Wrong Sec-Fetch-Mode header")

def commonUncredentialedRequestCheck(request):
  if len(request.cookies) > 0:
    return (535, [], "Cookie should not be sent to this endpoint")
  if request.headers.get(b"Sec-Fetch-Site") != b"cross-site":
    return (536, [], "Wrong Sec-Fetch-Site header")

def commonCredentialedRequestCheck(request):
  if request.cookies.get(b"cookie") != b"1":
    return (537, [], "Missing cookie")

def commonPostCheck(request):
  if not request.headers.get(b"Origin"):
    return (540, [], "Missing Origin")
  if request.method != "POST":
    return (541, [], "Method is not POST")
  if request.headers.get(b"Content-Type") != b"application/x-www-form-urlencoded":
    return (542, [], "Wrong Content-Type")
  if not request.POST.get(b"client_id"):
    return (543, [], "Missing 'client_id' POST parameter")

def manifestCheck(request):
  common_error = commonCheck(request)
  if (common_error):
    return common_error
  common_uncredentialed_error = commonUncredentialedRequestCheck(request)
  if (common_uncredentialed_error):
    return common_uncredentialed_error

  if request.headers.get(b"Origin"):
    return (539, [], "Should not have Origin")

def clientMetadataCheck(request):
  if (request.GET.get(b'skip_checks', b'0') != b'1'):
    common_error = commonCheck(request)
    if (common_error):
      return common_error
    common_uncredentialed_error = commonUncredentialedRequestCheck(request)
    if (common_uncredentialed_error):
      return common_uncredentialed_error

    if not request.headers.get(b"Origin"):
      return (540, [], "Missing Origin")

def accountsCheck(request):
  common_error = commonCheck(request)
  if (common_error):
    return common_error
  common_credentialed_error = commonCredentialedRequestCheck(request)
  if (common_credentialed_error):
    return common_credentialed_error

  if request.headers.get(b"Origin"):
    return (539, [], "Should not have Origin")

def tokenCheck(request):
  common_error = commonCheck(request, b"cors")
  if (common_error):
    return common_error
  common_credentialed_error = commonCredentialedRequestCheck(request)
  if (common_credentialed_error):
    return common_credentialed_error
  # The value of the Sec-Fetch-Site header can vary depending on the IdP origin
  # but it should not be 'none'.
  if request.headers.get(b"Sec-Fetch-Site") == b"none":
    return (538, [], "Wrong Sec-Fetch-Site header")

  post_error = commonPostCheck(request)
  if (post_error):
    return post_error

  if not request.POST.get(b"account_id"):
    return (544, [], "Missing 'account_id' POST parameter")
  if not request.POST.get(b"disclosure_text_shown"):
    return (545, [], "Missing 'disclosure_text_shown' POST parameter")
  if not request.headers.get(b"Origin"):
    return (540, [], "Missing Origin")

def revokeCheck(request):
  common_error = commonCheck(request, b"cors")
  if (common_error):
    return common_error

  common_credentialed_error = commonCredentialedRequestCheck(request)
  if (common_credentialed_error):
    return common_credentialed_error
  # The value of the Sec-Fetch-Site header can vary depending on the IdP origin
  # but it should not be 'none'.
  if request.headers.get(b"Sec-Fetch-Site") == b"none":
    return (538, [], "Wrong Sec-Fetch-Site header")

  post_error = commonPostCheck(request)
  if (post_error):
    return post_error

  if not request.POST.get(b"account_hint"):
    return (544, [], "Missing 'account_hint' POST parameter")
