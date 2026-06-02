# élève.
utf8_subdomain = b"Domain=\xC3\xA9\x6C\xC3\xA8\x76\x65."
# élève。
utf8_dot_subdomain = b"Domain=\xC3\xA9\x6C\xC3\xA8\x76\x65\xE3\x80\x82"
# élève.
punycode_subdomain = b"Domain=xn--lve-6lad."
# ÿlève.
wrong_utf8_subdomain = b"Domain=\xC3\xBF\x6C\xC3\xA8\x76\x65."
# ÿlève.
wrong_punycode_subdomain = b"Domain=xn--lve-6la7i."
# élève with invalid FF byte at the end
invalid_byte_subdomain = b"Domain=\xC3\xA9\x6C\xC3\xA8\x76\x65\xFF."

def main(request, response):
  host = request.GET.get(b"host")

  if b"set-utf8" in request.GET:
    response.headers.append(b"Set-Cookie", b"utf8=set;" + utf8_subdomain + host)
    response.content = "set"
  if b"set-utf8-dot" in request.GET:
    response.headers.append(b"Set-Cookie", b"utf8-dot=set;" + utf8_dot_subdomain + host)
    response.content = "set"
  elif b"set-wrong-utf8" in request.GET:
    response.headers.append(b"Set-Cookie", b"wrong-utf8=set;" + wrong_utf8_subdomain + host)
    response.content = "set"
  elif b"set-punycode" in request.GET:
    response.headers.append(b"Set-Cookie", b"punycode=set;" + punycode_subdomain + host)
    response.content = "set"
  elif b"set-wrong-punycode" in request.GET:
    response.headers.append(b"Set-Cookie", b"wrong-punycode=set;" + wrong_punycode_subdomain + host)
    response.content = "set"
  elif b"set-invalid-byte" in request.GET:
    response.headers.append(b"Set-Cookie", b"invalid-byte=set;" + invalid_byte_subdomain + host)
    response.content = "set"

  elif b"get" in request.GET:
    if b"Cookie" in request.headers:
      response.content = request.headers[b"Cookie"]
    else:
      response.content = "no cookies"

  elif b"delete-utf8" in request.GET:
    response.headers.append(b"Set-Cookie", b"utf8=unset;Max-Age=0;" + utf8_subdomain + host)
    response.content = "delete"
  elif b"delete-utf8-dot" in request.GET:
    response.headers.append(b"Set-Cookie", b"utf8-dot=unset;Max-Age=0;" + utf8_dot_subdomain + host)
    response.content = "delete"
  elif b"delete-wrong-utf8" in request.GET:
    response.headers.append(b"Set-Cookie", b"wrong-utf8=unset;Max-Age=0;" + wrong_utf8_subdomain + host)
    response.content = "delete"
  elif b"delete-punycode" in request.GET:
    response.headers.append(b"Set-Cookie", b"punycode=unset;Max-Age=0;" + punycode_subdomain + host)
    response.content = "delete"
  elif b"delete-wrong-punycode" in request.GET:
    response.headers.append(b"Set-Cookie", b"wrong-punycode=unset;Max-Age=0;" + wrong_punycode_subdomain + host)
    response.content = "delete"
  elif b"delete-invalid-byte" in request.GET:
    response.headers.append(b"Set-Cookie", b"invalid-byte=unset;Max-Age=0;" + invalid_byte_subdomain + host)
    response.content = "delete"

  response.headers.append(b"Content-Type", b"text/plain")
