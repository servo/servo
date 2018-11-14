# Always redirects to no-policy-pass.png.
def main(request, response):
  return 302, [("Location", "no-policy-pass.png")], ""
