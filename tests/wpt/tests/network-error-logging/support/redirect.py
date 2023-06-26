# Always redirects to no-policy-pass.png.
def main(request, response):
  return 302, [(b"Location", b"no-policy-pass.png")], u""
